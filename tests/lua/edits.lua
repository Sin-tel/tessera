-- Tool edits that are in progress while the project changes underneath them (undo, tuning changes).
-- Drives the real canvas and tools, without drawing.
-- cargo run -- --test-run tests/lua/edits.lua

local Canvas = require("views/canvas")
local Ui = require("ui/ui")
local edit = require("tools/edit")
local pen = require("tools/pen")
local presets = require("default.tuning_presets")
local tuning = require("tuning")

-- icons need the graphics context, the canvas only stores them
tessera.icon = tessera.icon or {}

local canvas

local function setup()
	project = {
		settings = { snap_pitch = 2, snap_time = 4, preview_notes = false },
		channels = { { notes = {}, visible = true } },
		time = { { 0, 2 } },
	}
	command.stack = {}
	command.index = 0
	tuning.system = nil
	tuning.center = nil
	tuning.load_project()
	selection.deselect()
	selection.ch_index = 1

	canvas = Canvas.new()
	-- so canvas:get_mouse() is the mouse position
	canvas.box = { x = -Ui.BORDER_SIZE, y = -(Ui.HEADER + Ui.BORDER_SIZE), focus = true }
	canvas.w, canvas.h = 1000, 1000
	mouse:load()
end

local function press(x, y)
	mouse.x, mouse.y = x, y
	mouse.button = 1
	mouse.is_down = true
	canvas:mousepressed()
end

local function move(x, y)
	mouse.x, mouse.y = x, y
	mouse.drag = true
	canvas:update()
end

local function release()
	canvas:mousereleased()
	mouse.is_down = false
	mouse.button = false
	mouse.drag = false
end

local function undo()
	command.undo()
	tuning.validate()
end

local function set_tuning(key)
	local def = util.clone(presets.tunings[key])
	for _, option in ipairs(tessera.tuning.notations(def)) do
		if option.recommended then
			def.notation = option.choice
			break
		end
	end
	local c = command.SetTuning.new(def)
	assert(c:run())
	command.register(c)
end

-- a note at C4, and where it is on screen
local function add_note()
	local note = { interval = tuning.new_interval(), time = 1, vel = 0.5, verts = { { 0, 0, 0 }, { 1, 0, 0 } } }
	command.run_and_register(command.NoteAdd.new({ { note } }))
	return note, canvas.transform:time(1), canvas.transform:pitch(60)
end

local tests = {}

-- The original bug: undo while drawing a note reverted the tuning under the pen.
tests["pen, undo tuning change"] = function()
	set_tuning("ji_7")
	canvas:select_tool(pen)
	press(300, 380)
	move(400, 380)
	-- cancels the note
	undo()
	assert(#project.channels[1].notes == 0, "pen note was not undone")
	undo()
	assert(project.settings.tuning.name == presets.default().name, "tuning was not undone")
	move(450, 380)
	release()
	assert(#project.channels[1].notes == 0, "pen added a note after it was finished")
end

tests["drag, undo tuning change"] = function()
	local note, x, y = add_note()
	set_tuning("ji_7")
	canvas:select_tool(edit)
	press(x, y)
	move(x + 50, y - 100)
	-- cancels the drag
	undo()
	assert(note.time == 1, "drag was not undone")
	undo()
	move(x + 100, y - 100)
	release()
	assert(note.time == 1, "note moved after the drag was finished")
end

-- Keyboard grab stays active until a click, so the tuning can change in another tab.
tests["grab, apply tuning"] = function()
	local note, x, y = add_note()
	selection.set({ [note] = true })
	mouse.x, mouse.y = x, y
	canvas:keypressed("g")
	move(x + 50, y - 100)
	set_tuning("ji_7")
	assert(not canvas.tool_active, "grab still active after tuning change")
	move(x + 100, y - 100)
	release()
	undo()
	undo()
	undo()
	assert(#project.channels[1].notes == 0)
end

-- Switching tools mid drag used to drop the drag without registering it.
tests["tab while dragging"] = function()
	local _, x, y = add_note()
	canvas:select_tool(edit)
	press(x, y)
	move(x + 50, y)
	canvas:keypressed("tab")
	move(x + 100, y)
	release()
	assert(command.stack[command.index].prev_state, "drag was not registered")
	assert(canvas.current_tool == pen, "tool did not switch after the drag")
end

tests["respell cycles"] = function()
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

tests["respell simplifies selection"] = function()
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

tests["auto simplify, transpose keys"] = function()
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

tests["auto simplify, drag"] = function()
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

-- Spellings that are about as simple as the simplest are kept.
tests["auto simplify, threshold"] = function()
	set_tuning("et_22")
	project.settings.auto_simplify = true
	local cost = function(p)
		return tuning.system:spelling_cost(p)
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

local names = {}
for name in pairs(tests) do
	table.insert(names, name)
end
table.sort(names)

local failures = 0
for _, name in ipairs(names) do
	setup()
	local ok, err = pcall(tests[name])
	if ok then
		ok, err = pcall(tuning.validate)
	end
	if ok then
		print("ok    " .. name)
	else
		failures = failures + 1
		print("FAIL  " .. name .. ": " .. tostring(err))
	end
end

return failures
