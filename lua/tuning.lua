local Notation = require("notation")
local log = require("log")
local scales = require("default.scales")
local tuning_presets = require("default.tuning_presets")

local tuning = {}

tuning.snap_labels = { "Diatonic", "Chromatic", "Fine" }

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

-- Load a tuning from definition table
function tuning.load(def)
	local ok, system = pcall(tessera.tuning.new, def, def.notation, scales.candidates)
	if not ok then
		log.error(system)
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

	-- Projections from notes to scale indices. Scales without one are looked up by pitch.
	tuning.maps = {}
	for i, t in ipairs(tuning.tables) do
		tuning.maps[t] = system:scale_map(i)
		if not tuning.maps[t] then
			log.warn(tuning.snap_labels[i] .. " scale has no projection, using nearest pitch.")
		end
	end
	tuning.center = pad(tuning.center or {})

	return true
end

-- Set the tuning for the current project.
function tuning.set(def)
	if not tuning.load(def) then
		return false
	end

	local name = def.name or def.subgroup
	log.info("Loading tuning: " .. name .. " (" .. def.notation.accidentals .. " accidentals)")
	project.settings.tuning = util.clone(def)

	-- TODO: do a proper conversion here

	-- fix up the number of coordinates
	for _, ch in ipairs(project.channels) do
		if ch.notes then
			for _, note in ipairs(ch.notes) do
				note.interval = pad(note.interval)
			end
		end
	end
	return true
end

-- Load the tuning from a project that was just loaded or created.
function tuning.load_project()
	if project.settings.tuning then
		if tuning.load(project.settings.tuning) then
			return
		end
		log.error("Failed to load tuning from project, using default.")
	end

	-- fallback to default if nothing works
	project.settings = tuning.load_default()
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

function tuning.get_center(p)
	return tuning.center
end
function tuning.set_center(p)
	tuning.center = util.clone(p)
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
local function nearest_index(t, r)
	local sp = get_scale_pitches(t)
	local o = tuning.generator_pitches[1]
	local n = #t

	local oct = math.floor(r / o)
	r = r - oct * o

	local best = 0
	local d_best = math.huge
	for i = 1, n do
		local d = math.abs(r - sp[i])
		if d < d_best then
			best = i - 1
			d_best = d
		elseif d > d_best then
			-- sorted, so it only gets worse
			break
		end
	end
	-- the unison in the next octave
	if math.abs(o - r) < d_best then
		best = n
	end
	return oct * n + best
end

-- Given some pitch p, find interval in current grid that is closest
function tuning.snap(p)
	local t = tuning.tables[project.settings.snap_pitch]
	assert(t)
	local rel = p - tuning.get_pitch(tuning.center)
	return tuning.from_table(t, nearest_index(t, rel) + tuning.get_index(t, tuning.center))
end

function tuning.snap_interval(f)
	local t = tuning.tables[project.settings.snap_pitch]
	local steps = tuning.get_index(t, f)
	return tuning.from_table(t, steps)
end

-- Look up interval in table, correcting for octave offsets
function tuning.from_table(t, i)
	local start = tuning.get_index(t, tuning.center)

	i = i - start
	local s = #t
	local oct = math.floor(i / s)
	i = i - oct * s
	local p = util.clone(t[i + 1])
	p[1] = p[1] + oct
	p = tuning.add(p, tuning.center)

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
		local intervals = tuning.system:simple_spellings(interval)
		local spellings = {}
		for _, v in ipairs(intervals) do
			table.insert(spellings, tuning.get_name(v))
		end
		spellings = util.dedup(spellings)

		local ratios = tuning.system:simple_ratios(interval)
		rows[i] = {
			-- name = tuning.get_name(interval),
			name = table.concat(spellings, " = "),
			ratio = table.concat(ratios, " ~ "),
			black = BLACK_KEYS[(midi - 60) % 12] or false,
		}
	end
	return rows
end

-- Index of interval p in scale t.
-- With a projection this is exact, otherwise it is the note closest in pitch.
function tuning.get_index(t, p)
	local map = tuning.maps[t]
	if map then
		local index = 0
		for i, v in ipairs(map) do
			index = index + v * (p[i] or 0)
		end
		return index
	end
	return nearest_index(t, tuning.get_relative_pitch(p))
end

-- Convert interval to pitch.
function tuning.get_pitch(p)
	return 60 + tuning.get_relative_pitch(p)
end

function tuning.get_relative_pitch(p)
	local f = 0
	for i, v in ipairs(p) do
		f = f + v * tuning.generator_pitches[i]
	end
	return f
end

-- Name of a note in the current tuning.
function tuning.get_name(p)
	if project.settings.relative_note_names then
		p = tuning.sub(p, tuning.center)
	elseif tuning.rank > 2 then
		-- only move by extra accidentals
		local new = {}
		for i = 1, tuning.rank do
			if i > 2 then
				new[i] = p[i] - tuning.center[i]
			else
				new[i] = p[i]
			end
		end
		p = new
	end

	return tuning.notation:name(p)
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
