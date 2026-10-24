local Notation = require("notation")
local log = require("log")
local scales = require("default.scales")
local tuning_presets = require("default.tuning_presets")

local tuning = {}

tuning.snap_labels = { "Diatonic", "Chromatic", "Fine" }
tuning.auto_simplify_cost = 7

local function unit(index)
	local v = tuning.new_interval()
	v[index] = 1
	return v
end

local function pad(p)
	local new = {}
	for i = 1, tuning.rank do
		new[i] = p[i] or 0
	end
	return new
end

tuning.pad = pad

-- Build the tuning system for a definition, without loading it.
local function new_system(def)
	return tessera.tuning.new(def, def.notation, scales.candidates)
end

-- Load a tuning from definition table
function tuning.load(def)
	local system = new_system(def)
	if not system then
		return false
	end

	def.notation = system:choice()
	tuning.system = system

	-- number of coordinates in a note
	tuning.rank = system:len()

	-- size in semitones of each coordinate
	tuning.generator_pitches = system:generator_pitches()

	tuning.notation = Notation.new(system:style())

	-- interval definitions
	tuning.octave = pad({ 1 })
	tuning.tone = pad({ -1, 2 }) -- whole tone
	tuning.semitone = pad({ 3, -5 }) -- diatonic semitone
	tuning.chroma = pad({ -4, 7 }) -- apotome, chromatic semitone
	if tuning.get_relative_pitch(tuning.chroma) < 0 then
		tuning.chroma = tuning.mul(tuning.chroma, -1)
	end

	local accidentals = system:style().accidentals

	-- Small steps for fine editing.
	-- If there are accidentals, they take precedence.
	-- Otherwise in an equal temperament, one step.
	tuning.comma = nil
	tuning.comma_alt = nil
	tuning.comma_alt2 = nil
	tuning.chroma_alt = nil
	if tuning.rank >= 3 then
		tuning.comma = unit(3)
		if tuning.rank >= 4 then
			tuning.comma_alt = unit(4)
		end
		if tuning.rank >= 5 then
			tuning.comma_alt2 = unit(5)
		end

		if #accidentals >= 1 and accidentals[1].ratio == "81/80" then
			-- 25/24 just chromatic semitone (6/5 - 5/4)
			tuning.chroma_alt = pad({ -4, 7, -2 })
		end
		if #accidentals >= 1 and accidentals[1].ratio == "64/63" then
			-- 28/27 septimal minor second (9/8 - 7/6)
			tuning.chroma_alt = pad({ 3, -5, -1 })
		end
	elseif system:step() then
		tuning.comma = pad(system:step())
	else
		-- pythagorean comma C - Dbb
		tuning.comma = pad({ 7, -12 })
		if tuning.get_relative_pitch(tuning.comma) < 0 then
			tuning.comma = tuning.mul(tuning.comma, -1)
		end
	end

	tuning.diatonic = system:scale(1)
	tuning.chromatic = system:scale(2)
	tuning.fine = system:scale(3)

	tuning.tables = { tuning.diatonic, tuning.chromatic, tuning.fine }

	-- Projections from notes to scale indices.
	tuning.maps = {}
	for i, t in ipairs(tuning.tables) do
		tuning.maps[t] = system:scale_map(i)
	end

	return true
end

function tuning.all_notes()
	local list = {}
	for _, ch in ipairs(project.channels) do
		for _, note in ipairs(ch.notes or {}) do
			table.insert(list, note)
		end
	end
	return list
end

-- Check that every note and the center fit the current tuning.
-- Notes from another tuning won't crash right away, but much later somewhere unrelated.
function tuning.validate()
	local function check(p, what)
		if #p ~= tuning.rank then
			error(what .. " has " .. #p .. " coordinates, tuning has " .. tuning.rank)
		end
	end
	for _, note in ipairs(tuning.all_notes()) do
		check(note.interval, "Note")
	end
	for _, c in ipairs(project.center) do
		check(c.interval, "Tuning center")
	end
end

function tuning.preview(def)
	local system = new_system(def)
	if not system or not tuning.system then
		return nil
	end
	return system:conversion(tuning.system)
end

-- Make the project's center fit the tuning that was just loaded. Older saves don't have one.
local function load_center()
	if not project.center or #project.center == 0 then
		project.center = { { time = 0.0, interval = {} } }
	end
	for _, c in ipairs(project.center) do
		c.interval = pad(c.interval)
	end
end

-- set the tuning for the current project, and convert all notes
function tuning.set(def)
	local old_system = tuning.system

	if not tuning.load(def) then
		return false
	end

	clipboard.clear()

	local name = def.name or def.subgroup
	log.info("Loading tuning: " .. name .. " (" .. def.notation.accidentals .. " accidentals)")
	project.settings.tuning = util.clone(def)

	if not old_system then
		load_center()
		return true
	end

	local notes = tuning.all_notes()
	local intervals = {}
	for i, note in ipairs(notes) do
		intervals[i] = note.interval
	end
	for _, c in ipairs(project.center) do
		table.insert(intervals, c.interval)
	end

	local new, changed = tuning.system:convert(old_system, intervals)
	for i, note in ipairs(notes) do
		note.interval = new[i]
	end
	for i, c in ipairs(project.center) do
		c.interval = new[#notes + i]
	end

	local kind = tuning.system:conversion(old_system)
	if changed > 0 then
		log.info("Converted notes (" .. kind .. "), " .. changed .. " changed spelling")
	end
	return true
end

-- Load the tuning from a project that was just loaded or created.
function tuning.load_project()
	if project.settings.tuning then
		if tuning.load(project.settings.tuning) then
			load_center()
			return
		end
		log.error("Failed to load tuning from project, using default.")
	end

	-- fallback to default if nothing works
	project.settings.tuning = tuning.load_default()
	project.center = nil
	load_center()
end

function tuning.load_default()
	local def = tuning_presets.default()
	assert(tuning.load(def), "Failed to load default tuning")
	return def
end

function tuning.new_interval()
	local new = {}
	for i = 1, tuning.rank do
		new[i] = 0
	end
	return new
end

-- The center is the root of the scales and of relative note names.
-- Stored in project.center, a list of { time, interval } sorted by time, so it can later change over the
-- timeline like key signatures. For now there's only ever one entry, at time 0.
function tuning.get_center()
	return project.center[1].interval
end

-- See command.SetCenter for undo.
function tuning.set_center(p)
	project.center[1].interval = util.clone(p)
end

-- Size in semitones of every note in a scale, above the unison. Cached per scale.
local scale_pitches = setmetatable({}, { __mode = "k" })
local function get_scale_pitches(t)
	local sp = scale_pitches[t]
	if not sp then
		sp = {}
		for i, v in ipairs(t) do
			sp[i] = tuning.get_relative_pitch(v)
		end
		scale_pitches[t] = sp
	end
	return sp
end

-- Index of the note in scale t that is closest to a pitch r, in semitones above the unison.
-- The scale repeats every octave and index 0 is the unison.
-- Notes are ordered by index, which need not be the order of their pitches, or even inside one octave.
local function nearest_index(t, r)
	local sp = get_scale_pitches(t)
	local o = tuning.generator_pitches[1]
	local n = #t

	local oct = math.floor(r / o)
	r = r - oct * o

	local best = 0
	local d_best = math.huge
	for k = -1, 1 do
		for i = 1, n do
			local d = math.abs(r - sp[i] - k * o)
			if d < d_best then
				best = k * n + i - 1
				d_best = d
			end
		end
	end
	return oct * n + best
end

-- Given some pitch p, find interval in current grid that is closest
function tuning.snap(p)
	local t = tuning.tables[project.settings.snap_pitch]
	assert(t)
	local rel = p - tuning.get_pitch(tuning.get_center())
	return tuning.from_table(t, nearest_index(t, rel) + tuning.get_index(t, tuning.get_center()))
end

function tuning.snap_interval(f)
	local t = tuning.tables[project.settings.snap_pitch]
	local steps = tuning.get_index(t, f)
	return tuning.from_table(t, steps)
end

-- Look up interval in table, correcting for octave offsets
function tuning.from_table(t, i)
	local start = tuning.get_index(t, tuning.get_center())

	i = i - start
	local s = #t
	local oct = math.floor(i / s)
	i = i - oct * s
	local p = util.clone(t[i + 1])
	p[1] = p[1] + oct
	p = tuning.add(p, tuning.get_center())

	return p
end

-- Indexed by midi number, middle C = midi note number 60.
-- Note: we currently assume #chromatic = 12 so this works.
function tuning.from_midi(n)
	return tuning.from_table(tuning.chromatic, n - 60)
end

local BLACK_KEYS = { [1] = true, [3] = true, [6] = true, [8] = true, [10] = true }

-- build midi note table
function tuning.input_map()
	local rows = {}
	for i = 1, #tuning.chromatic + 1 do
		local midi = 59 + i
		local interval = tuning.from_midi(midi)
		-- relative to the center like the name
		local ratios = tuning.system:simple_ratios(tuning.to_centered(interval))
		local prefix, name = tuning.get_name_parts(interval)
		rows[i] = {
			prefix = prefix,
			name = name,
			ratio = table.concat(ratios, " ~ "),
			black = BLACK_KEYS[(midi - 60) % 12] or false,
		}
	end
	return rows
end

-- Index of interval p in scale t, by its projection.
function tuning.get_index(t, p)
	local index = 0
	for i, v in ipairs(tuning.maps[t]) do
		index = index + v * p[i]
	end
	return index
end

-- Convert interval to pitch.
function tuning.get_pitch(p)
	return 60 + tuning.get_relative_pitch(p)
end

function tuning.get_relative_pitch(p)
	local f = 0
	for i = 1, tuning.rank do
		f = f + p[i] * tuning.generator_pitches[i]
	end
	return f
end

-- How far the center moves the written notes.
-- With relative note names that's all of it, otherwise only the extra accidentals.
local function center_offset()
	local c = tuning.get_center()
	if project.settings.relative_note_names then
		return c
	end
	local offset = tuning.new_interval()
	for i = 3, tuning.rank do
		offset[i] = c[i]
	end
	return offset
end

-- Coordinates relative to the center: how a note is written. Names and spellings work on these.
function tuning.to_centered(p)
	return tuning.sub(p, center_offset())
end

-- Inverse of to_centered.
function tuning.from_centered(p)
	return tuning.add(p, center_offset())
end

-- The best few ways to write a note, best first, and the index of p in that list if it's one of them.
-- Spelled as written, relative to the center.
local function respellings(p)
	local centered = tuning.to_centered(p)
	-- Compared by how they're written, since different spellings can be drawn the same (half sharps).
	local list = {}
	local names = {}
	for _, s in ipairs(tuning.system:respellings(centered)) do
		local name = tuning.notation:name(s)
		if not util.find(names, name) then
			table.insert(list, tuning.from_centered(s))
			table.insert(names, name)
		end
	end
	return list, util.find(names, tuning.notation:name(centered))
end

-- Next way to write the same note, cycling through the best few.
-- Unchanged if there's only one way to write it.
function tuning.respell(p)
	local list, i = respellings(p)
	if i then
		return list[i % #list + 1]
	end
	-- not one of the best ones
	return list[1]
end

-- Simplest way to write a note.
function tuning.simplify(p)
	return respellings(p)[1]
end

-- Moved notes written more simply, if the auto simplify setting is on.
-- All by the same step that doesn't change the pitch, so a chord stays the same chord, and only when that saves
-- more than auto_simplify_cost per note on average. Costs are as written, relative to the center.
local function auto_simplify(intervals)
	if not project.settings.auto_simplify or #intervals == 0 then
		return intervals
	end
	local centered = util.map(intervals, tuning.to_centered)
	return util.map(tuning.system:simplify_chord(centered, tuning.auto_simplify_cost), tuning.from_centered)
end

-- A single moved note.
function tuning.auto_simplify(p)
	return auto_simplify({ p })[1]
end

-- Moved notes, in place.
function tuning.auto_simplify_notes(notes)
	local intervals = {}
	for i, note in ipairs(notes) do
		intervals[i] = note.interval
	end
	local simplified = auto_simplify(intervals)
	for i, note in ipairs(notes) do
		note.interval = simplified[i]
	end
end

-- Respell notes in place.
-- A single note cycles. Several notes are simplified, unless they all are already, then they cycle.
-- That way, respelling a selection first cleans it up.
function tuning.respell_notes(notes)
	local simplify = false
	if #notes > 1 then
		for _, note in ipairs(notes) do
			local _, i = respellings(note.interval)
			if i ~= 1 then
				simplify = true
				break
			end
		end
	end

	for _, note in ipairs(notes) do
		if simplify then
			note.interval = tuning.simplify(note.interval)
		else
			note.interval = tuning.respell(note.interval)
		end
	end
end

-- Name of a note in the current tuning.
function tuning.get_name(p)
	return tuning.notation:name(tuning.to_centered(p))
end

-- Name of a note, split in the part before the letter and the rest. See Notation:name_parts.
function tuning.get_name_parts(p)
	return tuning.notation:name_parts(tuning.to_centered(p))
end

-- basic arithmetic functions

function tuning.add(a, b)
	-- add two pitches a and b
	local new = {}
	for i = 1, tuning.rank do
		new[i] = a[i] + b[i]
	end
	return new
end

function tuning.sub(a, b)
	-- subtract b from a
	local new = {}
	for i = 1, tuning.rank do
		new[i] = a[i] - b[i]
	end
	return new
end

function tuning.mul(a, b)
	-- multiply pitch a by scalar b
	local new = {}
	for i = 1, tuning.rank do
		new[i] = a[i] * b
	end
	return new
end

function tuning.eq(a, b)
	-- check equality for a and b
	for i = 1, tuning.rank do
		if a[i] ~= b[i] then
			return false
		end
	end
	return true
end

return tuning
