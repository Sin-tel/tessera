local log = require("log")
local tuning_presets = require("default.tuning_presets")
local tuning = {}

local MAX_RANK = 5

tuning.snap_labels = { "Diatonic", "Chromatic", "Fine" }

tuning.systems = {
	"meantone",
	-- "meantone_quarter",
	-- "flattone",
	"archytas",
	-- "mavila",
	"porcupine",
	"diaschismic",
	-- "semaphore",
	"slendric",
	"ji_5",
	"ji_7",
	"ji_11",
	"septal",
	"marvel",
	"pele_7",
	"et_19",
	"et_31",
	"et_34",
	"et_41",
}

tuning.notation_styles = { "ups", "heji", "johnston" }

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
		if v > 11 then
			assert(f[i] == nil or f[i] == 0, "Only up to 11-limit supported.")
		end
	end

	local p = tuning.new_interval()

	-- 5-limit inverse mapping:
	--  [ 1  1  0  4  4]
	--  [ 0  1  4 -2 -1]
	--  [ 0  0 -1  0  0]
	--  [ 0  0  0 -1  0]
	--  [ 0  0  0  0  1]
	p[1] = f[1] + f[2] + 4 * f[4] + 4 * f[5]
	p[2] = f[2] + 4 * f[3] - 2 * f[4] - f[5]
	p[3] = -f[3]
	p[4] = -f[4]
	p[5] = f[5]
	return p
end

local function change_basis_inv(f)
	local p = {}
	for i = 1, MAX_RANK do
		f[i] = f[i] or 0
	end

	-- 5-limit mapping (2/1, 3/2, 81/80, 64/63, 33/32):
	-- 	[ 1 -1 -4  6 -5]
	--  [ 0  1  4 -2  1]
	--  [ 0  0 -1  0  0]
	--  [ 0  0  0 -1  0]
	--  [ 0  0  0  0  1]

	p[1] = f[1]
	p[2] = -f[1] + f[2]
	p[3] = -4 * f[1] + 4 * f[2] - f[3]
	p[4] = 6 * f[1] - 2 * f[2] - f[4]
	p[5] = -5 * f[1] + f[2] + f[5]
	return p
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

	return scale
end

function tuning.load(key)
	local settings = tuning_presets[key]
	if not settings then
		log.error("Could not find tuning: " .. key)
		return
	end
	tuning.info = settings.info
	tuning.key = key
	tuning.type = settings.type

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

	-- use 81/80 for ups/downs by default
	tuning.ups_index = 3

	if settings.fine then
		if type(settings.fine) == "table" then
			tuning.fine = tuning.generate_scale(settings.fine[1], settings.fine[2])
		elseif type(settings.fine) == "string" then
			tuning.fine = parse_scale(tuning_presets.scales[settings.fine])
		else
			assert(false, settings.fine)
		end
	end

	if tuning.type == "meantone" then
		assert(tuning.rank == 2)
		-- Pythagorean comma / diesis
		tuning.comma = { 7, -12 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = tuning.generate_scale(12, 4)
		if not settings.fine then
			tuning.fine = tuning.generate_scale(31, 13)
		end
	elseif tuning.type == "mavila" then
		assert(tuning.rank == 2)
		-- chroma is flipped
		tuning.chroma = { 4, -7 }
		-- 16et comma
		tuning.comma = { -9, 16 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = parse_scale(tuning_presets.scales.mavila_12)
		if not settings.fine then
			tuning.fine = tuning.generate_scale(16)
		end
	elseif tuning.type == "pyth" then
		assert(tuning.rank == 2)
		-- comma is flipped in pythagorean systems
		tuning.comma = { -7, 12 }

		tuning.diatonic = tuning.generate_scale(7, 1)
		tuning.chromatic = tuning.generate_scale(12, 4)
		-- tuning.fine = tuning.generate_scale(29, 11)
		if not settings.fine then
			tuning.fine = tuning.generate_scale(17, 6)
		end
	elseif tuning.type == "ji_5" or tuning.type == "ji_7" or tuning.type == "ji_11" then
		-- assert(tuning.rank == 3)
		-- 81/80
		tuning.comma = { 0, 0, 1 }

		-- 25/24
		tuning.chroma_alt = { -4, 7, -2 }

		if tuning.type == "ji_7" or tuning.type == "ji_11" then
			tuning.comma_alt = { 0, 0, 0, 1 }
		end
		if tuning.type == "ji_11" then
			tuning.comma_alt2 = { 0, 0, 0, 0, 1 }
		end

		tuning.diatonic = parse_scale(tuning_presets.scales.zarlino)
		tuning.chromatic = parse_scale(tuning_presets.scales.duodene)
		if not settings.fine then
			tuning.fine = parse_scale(tuning_presets.scales.ji_5_22)
		end
	elseif tuning.type == "septal" then
		assert(tuning.rank == 4)
		-- 64/63
		tuning.ups_index = 4
		tuning.comma = { 0, 0, 0, 1 }
		tuning.diatonic = parse_scale(tuning_presets.scales.septal_7)
		tuning.chromatic = parse_scale(tuning_presets.scales.septal_12)
		if not settings.fine then
			tuning.fine = parse_scale(tuning_presets.scales.septal_36)
		end
	else
		assert(false, "Unknown tuning type: " .. tuning.type)
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
			math.floor(ratio_to_pitch(7) * n / 12 + 0.5),
			math.floor(ratio_to_pitch(11) * n / 12 + 0.5),
		})
	end

	util.pprint(tuning.maps)

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

local function get_name_johnston(p)
	local n_i = (p[2] + 1)
	local n_f = n_i % #tuning.circle_of_fifths
	local nominal = tuning.circle_of_fifths[n_f + 1]
	local sharps = math.floor(n_i / #tuning.circle_of_fifths)

	local acc = ""

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

	-- 25/24 = vv#
	local plus = sharps * 2

	-- A, E, B need extra +
	if n_f > 3 then
		plus = plus + 1
	end

	if tuning.rank >= 4 then
		-- 36/35 = ^7
		acc = acc .. accidental(p[4], "p", "q")
		plus = plus - p[4]
	end
	if tuning.rank >= 5 then
		-- 33/32 works the same
		-- re-uses arrows from HEJI
		acc = acc .. accidental(p[5], "r", "s")
	end

	-- add plus/minus last
	if tuning.rank >= 3 then
		-- 81/80 = +
		plus = plus + p[3]
		acc = acc .. accidental(plus, "l", "m")
	end

	return nominal .. acc
end

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

	local notation_style = project.settings.notation_style

	if notation_style == "johnston" then
		return get_name_johnston(p)
	end

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
		if notation_style == "ups" then
			acc_pre = acc_pre .. accidental(p[tuning.ups_index], "w", "v")
		end

		if notation_style == "heji" and not (notation_style == "ups" and tuning.ups_index == 3) then
			acc = acc .. accidental(p[3], "r", "s")
		elseif notation_style == "plus" then
			acc = acc .. accidental(p[3], "l", "m")
		end
	end

	if tuning.rank >= 4 then
		-- septimal comma (64/63)
		if not (notation_style == "ups" and tuning.ups_index == 4) then
			acc = acc .. accidental(p[4], "o", "n")
		end
	end

	if tuning.rank >= 5 then
		-- half sharp / flat (33/32)
		if not (notation_style == "ups" and tuning.ups_index == 5) then
			acc = acc .. accidental(p[5], "h", "e")
		end
	end

	return acc_pre .. nominal .. acc
end

-- generate well-formed scale
-- n = scale size (nr. of generators)
-- offset = nr. of generators down from root
function tuning.generate_scale(n, offset)
	-- Heuristic scale size: equal number up/down if center is D
	offset = offset or math.floor(0.5 + ((n - 1) / 2) - 2)
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
