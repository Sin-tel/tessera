-- Spelling in the canvas: respelling, auto simplify and the tuning center.
-- cargo run -- --test-run tests/lua/tuning.lua

local edit = require("tools/edit")
local tuning = require("tuning")

local h = dofile("tests/lua/lib/harness.lua")
local press, move, release, undo, set_tuning, add_note = h.press, h.move, h.release, h.undo, h.set_tuning, h.add_note

local tests = {}

tests["respell cycles"] = function(canvas)
	set_tuning("et_31")
	local note = add_note()
	local original = note.interval
	local names = { tuning.get_name(original) }
	selection.set({ [note] = true })
	modifier_keys.shift = true
	for i = 1, 3 do
		assert(canvas:keypressed("r"), "shift+r not handled")
		names[i + 1] = tuning.get_name(note.interval)
		assert(math.abs(tuning.get_pitch(note.interval) - 60) < 1e-9, "respell changed pitch")
	end
	modifier_keys.shift = false
	print("      " .. table.concat(names, " -> "))
	assert(names[2] ~= names[1], "respell did nothing")
	assert(tuning.eq(note.interval, original), "did not cycle back after 3")
	undo()
	undo()
	assert(tuning.get_name(note.interval) == names[2], "undo did not step back")
end

tests["respell simplifies selection"] = function(canvas)
	set_tuning("et_31")
	local a = add_note()
	local b = add_note()
	local simplest = util.clone(a.interval)
	local second = tuning.respell(simplest)
	b.interval = util.clone(second)
	selection.set({ [a] = true, [b] = true })
	modifier_keys.shift = true
	canvas:keypressed("r")
	assert(tuning.eq(a.interval, simplest) and tuning.eq(b.interval, simplest), "mixed selection not simplified")
	canvas:keypressed("r")
	assert(tuning.eq(a.interval, second) and tuning.eq(b.interval, second), "simplest selection did not cycle")
	canvas:keypressed("r")
	assert(tuning.eq(a.interval, simplest) and tuning.eq(b.interval, simplest), "did not simplify again")
	modifier_keys.shift = false
end

tests["auto simplify, transpose keys"] = function(canvas)
	set_tuning("et_31")
	local note = add_note()
	selection.set({ [note] = true })
	local expected = tuning.get_pitch(note.interval) + 2 * tuning.get_relative_pitch(tuning.chroma)

	-- C## isn't the simplest way to write it in 31et
	modifier_keys.ctrl = true
	canvas:keypressed("up")
	canvas:keypressed("up")
	assert(not tuning.eq(note.interval, tuning.simplify(note.interval)), "simplified while off")

	project.settings.auto_simplify = true
	canvas:keypressed("up")
	canvas:keypressed("down")
	modifier_keys.ctrl = false
	assert(math.abs(tuning.get_pitch(note.interval) - expected) < 1e-9, "wrong pitch")
	assert(tuning.eq(note.interval, tuning.auto_simplify(note.interval)), "not simplified")
end

tests["auto simplify, drag"] = function(canvas)
	set_tuning("et_31")
	project.settings.auto_simplify = true
	local note, x, y = add_note()
	-- a deliberately complex spelling of C
	local complex = tuning.respell(tuning.respell(note.interval))
	note.interval = util.clone(complex)

	-- only in time: keeps the spelling
	canvas:select_tool(edit)
	press(x, y)
	move(x + 100, y)
	release()
	assert(tuning.eq(note.interval, complex), "respelled without a pitch change")

	-- in pitch
	x = canvas.transform:time(note.time)
	press(x, y)
	move(x, y - 50)
	move(x, y - 51)
	release()
	assert(tuning.get_pitch(note.interval) > 60, "did not move up")
	assert(tuning.eq(note.interval, tuning.auto_simplify(note.interval)), "not simplified")
end

-- A chord is respelled as a whole: the intervals between its notes keep their spelling.
tests["auto simplify, chord"] = function(canvas)
	set_tuning("et_31")
	project.settings.auto_simplify = true
	local chord = { tuning.new_interval(), tuning.pad({ -2, 4 }), tuning.pad({ -1, 1 }) }
	local notes = {}
	for i, p in ipairs(chord) do
		notes[i] = add_note()
		notes[i].interval = util.clone(p)
	end
	selection.set({ [notes[1]] = true, [notes[2]] = true, [notes[3]] = true })

	local function written(i)
		return tuning.sub(notes[i].interval, notes[1].interval)
	end
	local respelled, per_note_breaks = false, false
	modifier_keys.ctrl = true
	for step = 1, 12 do
		-- what simplifying each note on its own would do
		local separate = {}
		for i, note in ipairs(notes) do
			local p = tuning.add(note.interval, tuning.chroma)
			separate[i] = tuning.auto_simplify(p)
			if not tuning.eq(separate[i], p) then
				respelled = true
			end
		end
		for i = 2, #notes do
			if not tuning.eq(tuning.sub(separate[i], separate[1]), chord[i]) then
				per_note_breaks = true
			end
		end

		canvas:keypressed("up")
		local expected = step * tuning.get_relative_pitch(tuning.chroma)
		for i = 1, #notes do
			assert(tuning.eq(written(i), chord[i]), "chord spelling changed at step " .. step)
			local pitch = tuning.get_pitch(notes[i].interval) - tuning.get_pitch(chord[i])
			assert(math.abs(pitch - expected) < 1e-9, "wrong pitch at step " .. step)
		end
	end
	modifier_keys.ctrl = false
	assert(respelled, "chord was never simplified")
	assert(per_note_breaks, "test doesn't cover a chord that per-note simplifying would break")
end

-- Spellings that are about as simple as the simplest are kept.
tests["auto simplify, threshold"] = function()
	set_tuning("et_22")
	project.settings.auto_simplify = true
	local cost = function(p)
		return tuning.system:spelling_cost(tuning.to_centered(p))
	end
	local kept, simplified = 0, 0
	for m = 60, 71 do
		local s = tuning.from_midi(m)
		for _ = 1, 3 do
			local best = tuning.simplify(s)
			local gain = cost(s) - cost(best)
			local result = tuning.auto_simplify(s)
			if gain > tuning.auto_simplify_cost then
				assert(tuning.eq(result, best), "not simplified: " .. tuning.get_name(s))
				simplified = simplified + 1
			else
				assert(tuning.eq(result, s), "respelled with gain " .. gain .. ": " .. tuning.get_name(s))
				if gain > 0 then
					kept = kept + 1
				end
			end
			s = tuning.respell(s)
		end
	end
	assert(kept > 0 and simplified > 0, "test doesn't cover both cases")
end

tests["set center"] = function(canvas)
	local note = add_note()
	note.interval = util.clone(tuning.tone)
	local default = tuning.new_interval()
	assert(tuning.eq(tuning.get_center(), default), "new project doesn't start at the default center")
	selection.set({ [note] = true })
	local n = command.index

	-- only the plain key
	modifier_keys.shift = true
	modifier_keys.any = true
	canvas:keypressed("c")
	modifier_keys.shift = false
	modifier_keys.any = false
	assert(command.index == n, "set center with a modifier")

	canvas:keypressed("c")
	assert(tuning.eq(tuning.get_center(), tuning.tone), "center not set")
	assert(tuning.get_center() ~= note.interval, "center shares the note's table")
	canvas:keypressed("c")
	assert(command.index == n + 1, "setting the same center registered again")

	-- nothing selected: back to default
	selection.deselect()
	canvas:keypressed("c")
	assert(tuning.eq(tuning.get_center(), default), "center not reset")
	undo()
	assert(tuning.eq(tuning.get_center(), tuning.tone), "reset not undone")
	undo()
	assert(tuning.eq(tuning.get_center(), default), "center not undone")
	command.redo()
	assert(tuning.eq(tuning.get_center(), tuning.tone), "center not redone")
end

-- The center is saved with the project, and filled in for older saves.
tests["center in project"] = function()
	set_tuning("et_31")
	tuning.set_center(tuning.tone)
	local saved = util.clone(project)

	project = util.clone(saved)
	project.center = nil
	tuning.load_project()
	assert(tuning.eq(tuning.get_center(), tuning.new_interval()), "missing center not filled in")

	project = util.clone(saved)
	tuning.load_project()
	assert(tuning.eq(tuning.get_center(), tuning.tone), "center not kept")
end

-- Only one center for now, but tuning changes already handle all of them.
tests["center over timeline"] = function()
	tuning.set_center(tuning.tone)
	table.insert(project.center, { time = 4.0, interval = util.clone(tuning.semitone) })
	local before = util.clone(project.center)
	set_tuning("ji_7")
	assert(#project.center == 2, "center entries lost")
	tuning.validate()
	undo()
	assert(tuning.eq(project.center[1].interval, before[1].interval), "first center not restored")
	assert(tuning.eq(project.center[2].interval, before[2].interval), "second center not restored")
	assert(project.center[2].time == 4.0)
end

tests["centered round trip"] = function()
	set_tuning("et_31")
	tuning.set_center(tuning.pad({ 2, -3, 1 }))
	for _, relative in ipairs({ true, false }) do
		project.settings.relative_note_names = relative
		for f = -8, 8 do
			for u = -2, 2 do
				local p = tuning.pad({ 0, f, u })
				assert(tuning.eq(tuning.from_centered(tuning.to_centered(p)), p), "round trip")
			end
		end
	end
end

-- Spellings are simple as written relative to the center: in F# major, A# reads as E and stays.
tests["simplify relative to center"] = function()
	set_tuning({ name = "12 equal", subgroup = "2.3.5", et = 12 })
	project.settings.relative_note_names = true
	local a_sharp = tuning.pad({ -5, 10 })
	local b_flat = tuning.pad({ 2, -2 })
	assert(math.abs(tuning.get_pitch(a_sharp) - tuning.get_pitch(b_flat)) < 1e-9)

	assert(tuning.eq(tuning.simplify(a_sharp), b_flat), "A# not simplified to Bb around C")

	tuning.set_center(tuning.pad({ -3, 6 }))
	assert(tuning.eq(tuning.simplify(a_sharp), a_sharp), "A# respelled in F#")
	assert(tuning.eq(tuning.simplify(b_flat), a_sharp), "Bb not simplified to A# in F#")
	assert(tuning.get_name(a_sharp) == "E", "A# in F# is named " .. tuning.get_name(a_sharp))
end

-- The input map's ratios are as written, like its names: with relative names the center is 1/1.
tests["input map ratios follow center"] = function()
	set_tuning("ji_5")
	tuning.set_center(tuning.tone)
	-- midi 62 plays the center
	assert(tuning.eq(tuning.from_midi(62), tuning.tone))

	project.settings.relative_note_names = true
	local row = tuning.input_map()[3]
	assert(row.ratio:find("^1/1"), "center is " .. row.ratio .. " with relative names")

	project.settings.relative_note_names = false
	row = tuning.input_map()[3]
	assert(row.ratio:find("^9/8"), "center is " .. row.ratio .. " with absolute names")
end

return h.run(tests)
