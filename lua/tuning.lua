local tuning_presets = require("default.tuning_presets")
local tuning = {}

tuning.snap_labels = { "Diatonic", "Chromatic", "Fine" }

local PRIMES = {
	2,
	3,
	5,
	7,
	11,
	13,
}

local function factorize(p, q)
	local _p, _q = p, q
	local f = {}
	for i, v in ipairs(PRIMES) do
		f[i] = 0
		while p % v == 0 do
			p = p / v
			f[i] = f[i] + 1
		end

		while q % v == 0 do
			q = q / v
			f[i] = f[i] - 1
		end
		if q == 1 and p == 1 then
			break
		end
	end
	assert(p == 1 and q == 1, "Prime decomposition failed for " .. _p .. "/" .. _q)
	return f
end

-- change from prime to pythagorean + accidental basis
-- TODO: generalize this for higher limits depending on required accidentals
local function change_basis(f)
	for i, v in ipairs(PRIMES) do
		f[i] = f[i] or 0
		if v > 5 then
			assert(f[i] == nil or f[i] == 0, "Only up to 5-limit supported.")
		end
	end

	local c = -f[3]
	local b = f[2] - 4 * c
	local a = f[1] + b + 4 * c
	return { a, b, c }
end

local function ratio_to_pitch(r)
	return 12.0 * math.log(r) / math.log(2)
end

local function parse_ratio(s)
	local p, q = s:match("(%d+)/(%d+)")
	assert(p)
	assert(q)
	return math.floor(p), math.floor(q)
end

local function parse_scale(s)
	local scale = {}

	-- add 1/1
	scale[1] = { 0, 0 }

	for i, str in ipairs(s) do
		if i == #s then
			-- assume octaves for now
			local p, q = parse_ratio(str)
			assert(p == 2, q == 1)
			break
		end

		local f = change_basis(factorize(parse_ratio(str)))
		table.insert(scale, f)
	end

	-- Should be already sorted, but can't hurt
	table.sort(scale, function(a, b)
		return tuning.get_relative_pitch(a) < tuning.get_relative_pitch(b)
	end)

	return scale
end

function tuning.load(key)
	local settings = tuning_presets[key]
	tuning.settings = settings

	-- load generators
	tuning.generators = {}
	tuning.rank = #settings.generators

	for i, v in ipairs(settings.generators) do
		-- Parse ratio if given as string
		if type(v) == "string" then
			local p, q = parse_ratio(v)
			tuning.generators[i] = ratio_to_pitch(p / q)
		end
	end

	-- interval definitions
	tuning.circle_of_fifths = { "F", "C", "G", "D", "A", "E", "B" }

	tuning.octave = { 1 }
	tuning.tone = { -1, 2 } -- whole tone
	tuning.semitone = { 3, -5 } -- diatonic semitone
	tuning.chroma = { -4, 7 } -- apotome, chromatic semitone

	if settings.type == "meantone" then
		assert(tuning.rank == 2)
		-- Pythagorean comma / diesis
		tuning.comma = { 7, -12 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = tuning.generate_scale(12, 4)
		tuning.fine = tuning.generate_scale(31, 12)
	elseif settings.type == "pyth" then
		assert(tuning.rank == 2)
		-- comma is flipped in pythagorean systems
		tuning.comma = { -7, 12 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = tuning.generate_scale(12, 4)
		tuning.fine = tuning.generate_scale(29, 11)
	elseif settings.type == "ji_5" then
		assert(tuning.rank == 3)
		-- 81/80
		tuning.comma = { 0, 0, 1 }
		tuning.diatonic = parse_scale(tuning_presets.scales.zarlino)
		tuning.chromatic = parse_scale(tuning_presets.scales.duodene)
		tuning.fine = parse_scale(tuning_presets.scales.ji_5_fine)
	else
		assert(false, "Unknown tuning type: " .. settings.type)
	end

	tuning.tables = { tuning.diatonic, tuning.chromatic, tuning.fine }

	util.pprint(tuning.tables)
	tuning.center = tuning.new_interval()
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

-- Given some pitch p, find interval in current grid that is closest
function tuning.snap(p)
	local t = tuning.tables[project.settings.snap_pitch]
	assert(t)
	local steps = math.floor(p * (#t / 12) + 0.5)
	return tuning.from_table(t, steps)
end

-- Look up interval in table, correcting for octave offsets
function tuning.from_table(t, i)
	local start = tuning.get_index(#t, tuning.center)

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

-- Project an interval to an n-note scale via linear mapping,
-- ignoring any extra accidentals.
function tuning.get_index(n, p)
	local s1 = math.floor(n * tuning.generators[1] / 12 + 0.5)
	local s2 = math.floor(n * tuning.generators[2] / 12 + 0.5)
	return s1 * p[1] + s2 * p[2]
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

-- TODO: generalize this to other systems
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

	-- factor 4/7 is because base note name does not change when altering by an apotome (#) which is [-4, 7]
	local o = p[1] + math.floor(p[2] * 4 / 7) + 4
	local n_i = (p[2] + 1)

	local sharps = math.floor(n_i / #tuning.circle_of_fifths)

	local acc = ""
	if sharps > 0 then
		if sharps % 2 == 1 then
			acc = "c" -- #
		end
		local double_sharps = math.floor(sharps / 2)
		-- x
		acc = acc .. string.rep("d", double_sharps)
	elseif sharps < 0 then
		local flats = -sharps
		if flats == 1 then
			acc = "a" --b
		elseif flats == 2 then
			acc = "e" --bb
		elseif flats == 3 then
			acc = "f" --bbb
		else
			local group = (flats - 1) % 3
			if group == 0 then
				acc = "ee" -- bb bb
			elseif group == 1 then
				acc = "ef" -- bb bbb
			else
				acc = "ff" -- bbb bbb
			end
			local triple_flats = math.floor((flats - 4) / 3)
			if triple_flats > 0 then
				--- bbb
				acc = acc .. string.rep("f", triple_flats)
			end
		end
	end

	if tuning.rank >= 3 then
		local plus = p[3]
		if plus > 0 then
			acc = acc .. string.rep("l", plus)
			-- acc = acc .. string.rep("j", plus)
		elseif plus < 0 then
			local minus = -plus
			acc = acc .. string.rep("m", minus)
			-- acc = acc .. string.rep("k", minus)
		end
	end

	local n = tuning.circle_of_fifths[n_i % #tuning.circle_of_fifths + 1]

	return n .. acc .. tostring(o)
end

-- generate well-formed scale
-- n = scale size (nr. of generators)
-- offset = nr. of generators down from root
function tuning.generate_scale(n, offset)
	local scale = {}
	local o = tuning.get_relative_pitch(tuning.octave)
	for i = 0, n - 1 do
		local note = { 0, i - offset }

		local p = tuning.get_relative_pitch(note)
		note[1] = -math.floor(p / o)
		table.insert(scale, note)
	end

	table.sort(scale, function(a, b)
		return tuning.get_relative_pitch(a) < tuning.get_relative_pitch(b)
	end)

	return scale
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
