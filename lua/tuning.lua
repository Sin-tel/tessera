local log = require("log")
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

	-- 5-limit map:
	-- 1  1  0
	-- 0  1  4
	-- 0  0 -1
	local a = f[1] + f[2]
	local b = f[2] + 4 * f[3]
	local c = -f[3]
	return { a, b, c }
end

local function change_basis_inv(f)
	-- invert and transpose
	local a = f[1]
	local b = -f[1] + f[2]
	local c = -4 * f[1] + 4 * f[2] - f[3]
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
	if not settings then
		log.error("Could not find tuning: " .. key)
		return
	end
	tuning.settings = settings
	tuning.key = key

	-- style of accidentals ("plus", "ups", "heji")
	tuning.acc_style = "ups"

	if project.settings then
		-- persist in save file
		project.settings.tuning_key = key
		log.info("Loading tuning: " .. key)
	end

	-- load generators
	tuning.generators = {}
	tuning.rank = #settings.generators

	for i, v in ipairs(settings.generators) do
		-- Parse ratio if given as string
		if type(v) == "string" then
			local p, q = parse_ratio(v)
			tuning.generators[i] = ratio_to_pitch(p / q)
		elseif type(v) == "number" then
			tuning.generators[i] = v
		else
			assert(false, "unsupported generator " .. v)
		end
	end

	-- interval definitions
	tuning.circle_of_fifths = { "F", "C", "G", "D", "A", "E", "B" }

	tuning.octave = { 1 }
	tuning.tone = { -1, 2 } -- whole tone
	tuning.semitone = { 3, -5 } -- diatonic semitone
	tuning.chroma = { -4, 7 } -- apotome, chromatic semitone

	if settings.fine then
		tuning.fine = tuning.generate_scale(settings.fine[1], settings.fine[2])
	end

	if settings.type == "meantone" then
		assert(tuning.rank == 2)
		-- Pythagorean comma / diesis
		tuning.comma = { 7, -12 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = tuning.generate_scale(12, 4)
		if not settings.fine then
			tuning.fine = tuning.generate_scale(31, 12)
		end
	elseif settings.type == "pyth" then
		assert(tuning.rank == 2)
		-- comma is flipped in pythagorean systems
		tuning.comma = { -7, 12 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = tuning.generate_scale(12, 4)
		-- tuning.fine = tuning.generate_scale(29, 11)
		if not settings.fine then
			tuning.fine = tuning.generate_scale(17, 6)
		end
	elseif settings.type == "ji_5" then
		assert(tuning.rank == 3)
		-- 81/80
		tuning.comma = { 0, 0, 1 }
		tuning.diatonic = parse_scale(tuning_presets.scales.zarlino)
		tuning.chromatic = parse_scale(tuning_presets.scales.duodene)
		if not settings.fine then
			tuning.fine = parse_scale(tuning_presets.scales.ji_5_fine)
		end
	else
		assert(false, "Unknown tuning type: " .. settings.type)
	end

	tuning.tables = { tuning.diatonic, tuning.chromatic, tuning.fine }

	-- pre-calculate projections onto ETs for scale lookups
	tuning.maps = {}
	for _, t in pairs(tuning.tables) do
		local n = #t
		tuning.maps[n] = change_basis_inv({
			math.floor(ratio_to_pitch(2) * n / 12 + 0.5),
			math.floor(ratio_to_pitch(3) * n / 12 + 0.5),
			math.floor(ratio_to_pitch(5) * n / 12 + 0.5),
		})
	end

	-- check if mappings are one-to-one
	for t_name, t in pairs(tuning.tables) do
		for i, v in ipairs(t) do
			if tuning.get_index(#t, v) + 1 ~= i then
				log.warn("Inconsistency in scale " .. t_name .. " index " .. i)
			end
		end
	end

	if not tuning.center then
		tuning.center = tuning.new_interval()
	end
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
	local p_start = tuning.get_pitch(tuning.center)
	local s_start = tuning.get_index(#t, tuning.center)
	local steps = math.floor((p - p_start) * (#t / 12) + 0.5)
	return tuning.from_table(t, steps + s_start)
end

function tuning.snap_interval(f)
	local t = tuning.tables[project.settings.snap_pitch]
	local steps = tuning.get_index(#t, f)
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

-- Project an interval to an n-note scale via linear mapping.
function tuning.get_index(n, p)
	local sum = 0.0
	for i = 1, tuning.rank do
		sum = sum + tuning.maps[n][i] * (p[i] or 0)
	end
	return sum
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

local function accidental(n, c_up, c_down)
	if n > 0 then
		return string.rep(c_up, n)
	elseif n < 0 then
		return string.rep(c_down, -n)
	else
		return ""
	end
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
	-- local o = p[1] + math.floor(p[2] * 4 / 7) + 4
	local n_i = (p[2] + 1)
	local nominal = tuning.circle_of_fifths[n_i % #tuning.circle_of_fifths + 1]
	local sharps = math.floor(n_i / #tuning.circle_of_fifths)

	local acc = ""
	local acc_pre = ""
	if sharps > 0 then
		if sharps % 2 == 1 then
			acc = acc .. "c" -- #
		end
		local double_sharps = math.floor(sharps / 2)
		acc = acc .. string.rep("d", double_sharps) -- x
	elseif sharps < 0 then
		local flats = -sharps
		acc = acc .. string.rep("a", flats)
	end

	if tuning.rank >= 3 then
		if tuning.acc_style == "heji" then
			acc = acc .. accidental(p[3], "r", "s")
		elseif tuning.acc_style == "ups" then
			acc_pre = acc_pre .. accidental(p[3], "w", "v")
		else
			acc = acc .. accidental(p[3], "l", "m")
		end
	end

	-- return acc_pre .. nominal .. acc .. tostring(o)
	return acc_pre .. nominal .. acc
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
