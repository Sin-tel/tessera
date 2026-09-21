local Notation = require("notation")
local log = require("log")
local tuning_presets = require("default.tuning_presets")

local tuning = {}

tuning.snap_labels = { "Diatonic", "Chromatic", "Fine" }

-- Half an apotome in semitones, the same bound that xen_utils uses for accidentals.
-- local MAX_ACCIDENTAL = 0.56842503

local function unit(index)
	local v = tuning.new_interval()
	v[index] = 1
	return v
end

-- Load a tuning from a definition table
function tuning.load(def)
	local system = tessera.tuning.new(def)

	tuning.def = def
	tuning.system = system

	-- number of coordinates in a note
	tuning.rank = system:len()

	-- size in semitones of each coordinate
	tuning.generators = system:pitches()

	-- build notation
	local info = tuning.system:get_notation_info()
	tuning.notation = Notation.new(info)

	-- interval definitions
	tuning.octave = { 1 }
	tuning.tone = { -1, 2 } -- whole tone
	tuning.semitone = { 3, -5 } -- diatonic semitone
	tuning.chroma = { -4, 7 } -- apotome, chromatic semitone
	if tuning.get_relative_pitch(tuning.chroma) < 0 then
		tuning.chroma = tuning.mul(tuning.chroma, -1)
	end

	-- Small steps for fine editing.
	-- If there are accidentals, they take precedence.
	-- Otherwise in an equal temperament, one step.
	-- Otherwise we use the first chain of fifths that closes up to an octave stack (pythagorean comma)
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

		-- TODO: fixme
		-- for an accidental of 81/80 the chromatic semitone is 25/24
		-- local i5 = tuning.accidental_index[5]
		-- if i5 then
		-- 	tuning.chroma_alt = util.clone(tuning.chroma)
		-- 	tuning.chroma_alt = tuning.add(tuning.chroma_alt, tuning.mul(unit(i5), -2))
		-- end
	elseif system:step() then
		tuning.comma = tuning.conform(system:step())
	else
		-- TODO: useful to keep?

		-- local o, f = tuning.generators[1], tuning.generators[2]
		-- for n = 2, 60 do
		-- 	local k = math.floor(n * f / o + 0.5)
		-- 	local d = k * o - n * f
		-- 	if math.abs(d) < MAX_ACCIDENTAL then
		-- 		local c = tuning.new_interval()
		-- 		c[1] = k
		-- 		c[2] = -n
		-- 		if d < 0 then
		-- 			c = tuning.mul(c, -1)
		-- 		end
		-- 		tuning.comma = c
		-- 		break
		-- 	end
		-- end

		-- pythagorean comma C - Dbb
		tuning.comma = tuning.conform({ 7, -12 })
		if tuning.get_relative_pitch(tuning.comma) < 0 then
			tuning.comma = tuning.mul(tuning.comma, -1)
		end
	end

	tuning.diatonic = system:scale(1)
	tuning.chromatic = system:scale(2)
	tuning.fine = system:scale(3)

	tuning.tables = { tuning.diatonic, tuning.chromatic, tuning.fine }
	tuning.center = tuning.conform(tuning.center or {})

	return true
end

-- Set the tuning for the current project.
function tuning.set(def)
	-- local n_accidentals_old = tuning.n_accidentals
	if not tuning.load(def) then
		return false
	end

	local n_accidentals = def.n_accidentals

	local name = def.name or def.subgroup
	log.info("Loading tuning: " .. name .. " (" .. n_accidentals .. " accidentals)")
	project.settings.tuning = util.clone(def)

	-- TODO: fixme
	-- do a proper conversion here

	-- fix up the number of coordinates
	for _, ch in ipairs(project.channels) do
		if ch.notes then
			for _, note in ipairs(ch.notes) do
				note.interval = tuning.conform(note.interval)
			end
		end
	end

	-- if n_changed > 0 then
	-- 	log.warn("Some information was lost by the conversion")
	-- end
	return true
end

-- Load the tuning from a project that was just loaded or created.
function tuning.load_project()
	local settings = project.settings

	if settings.tuning_legacy then
		-- Save file from before tuning definitions. We can't tell what it meant,
		-- so it stays on the default until a new tuning is picked.
		log.warn('Project was saved with tuning "' .. settings.tuning_legacy .. '". Please pick a new tuning.')
	end

	if settings.tuning then
		if tuning.load(settings.tuning) then
			return
		end
		log.error("Failed to load tuning from project, using default.")
	end

	settings.tuning = tuning_presets.default()
	tuning.load(settings.tuning)
end

function tuning.load_default()
	tuning.load(tuning_presets.default())
end

function tuning.new_interval()
	local new = {}
	for i = 1, tuning.rank do
		new[i] = 0
	end
	return new
end

-- Return a copy of the interval with exactly the right number of coordinates.
function tuning.conform(p)
	local new = {}
	for i = 1, tuning.rank do
		new[i] = p[i] or 0
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
	local o = tuning.generators[1]
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
	local p = t[i + 1]

	local new = tuning.new_interval()
	for k = 1, tuning.rank do
		new[k] = (p[k] or 0)
	end
	new[1] = new[1] + oct

	new = tuning.add(new, tuning.center)

	return new
end

-- Indexed by midi number, middle C = midi note number 60.
-- Note: we currently assume #chromatic = 12 so this works.
function tuning.from_midi(n)
	return tuning.from_table(tuning.chromatic, n - 60)
end

-- Index of the note in scale t that is closest to interval p.
function tuning.get_index(t, p)
	return nearest_index(t, tuning.get_relative_pitch(p))
end

-- Convert interval to pitch.
function tuning.get_pitch(p)
	return 60 + tuning.get_relative_pitch(p)
end

function tuning.get_relative_pitch(p)
	assert(p)
	local f = 0
	for i, v in ipairs(p) do
		f = f + v * (tuning.generators[i] or 0)
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
				new[i] = (p[i] or 0) - (tuning.center[i] or 0)
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
		new[i] = (a[i] or 0) + (b[i] or 0)
	end
	return new
end

function tuning.sub(a, b)
	-- subtract b from a
	local new = {}
	for i = 1, tuning.rank do
		new[i] = (a[i] or 0) - (b[i] or 0)
	end
	return new
end

function tuning.mul(a, b)
	-- multiply pitch a by scalar b
	local new = {}
	for i = 1, tuning.rank do
		new[i] = (a[i] or 0) * b
	end
	return new
end

function tuning.eq(a, b)
	-- check equality for a and b
	for i = 1, tuning.rank do
		if (a[i] or 0) ~= (b[i] or 0) then
			return false
		end
	end
	return true
end

return tuning
