--[[
pitch system is based on a diatonic (octave/fifth or period/gen) generated scale (can be any rank 2 MOS)
which is notated with alphabetic characters + b/#
there is also 6 extra accidental pairs
	-> total max 8 dimensions (e.g. up to 19-limit JI)
]]

-- TODO: load tuning info from file

local tuning = {}
-- local ratio = util.ratio

-- 11-limit JI table
-- tuning.rank = 5
-- tuning.generators = {
-- 	ratio(2/1),
-- 	ratio(3/2),
-- 	ratio(81/80),
-- 	ratio(64/63),
-- 	ratio(33/32),
-- }

-- 5-limit JI
-- tuning.rank = 3
-- tuning.generators = {
-- 	ratio(2 / 1),
-- 	ratio(3 / 2),
-- 	ratio(81 / 80),
-- }

-- Argent / Pele / Hemifamity (7-limit)
-- 5/4 = E-
-- 7/4 = Bb-
-- tuning.rank = 3
-- tuning.generators = {
-- 	11.9972,
-- 	7.02664,
-- 	0.24493,
-- }

-- Pele (11-limit)
-- 5/4 = E-
-- 7/4 = Bb-
-- 11/8 = Gb-
-- tuning.rank = 3
-- tuning.generators = {
-- 	11.99542,
-- 	7.03011,
-- 	0.25316,
-- }

-- Akea (11-limit)
-- 5/4 = E-
-- 7/4 = Bb-
-- 11/8 = F++
-- tuning.rank = 3
-- tuning.generators = {
-- 	12.0014,
-- 	7.02924,
-- 	0.26236,
-- }

-- 41edo rank3
-- tuning.rank = 3
-- tuning.generators = {
-- 	12,
-- 	12 * 24 / 41,
-- 	12 * 1 / 41,
-- }

-- septimal meantone WE
-- 7/4 = A#
-- 11/8 = Ex or Gbb
tuning.rank = 2
tuning.generators = {
	12.01236,
	6.97212,
}

-- 31edo rank 2
-- tuning.rank = 2
-- tuning.generators = {
-- 	12.0,
-- 	12.0 * 18.0 / 31.0,
-- }

-- flattone
-- 7/4 = Bbb
-- 11/8 = F#
-- tuning.rank = 2
-- tuning.generators = {
-- 	12.02062,
-- 	6.94545,
-- }

-- archytas
-- tuning.rank = 2
-- tuning.generators = {
-- 	11.96955,
-- 	7.07522,
-- }

-- 29edo
-- tuning.rank = 2
-- tuning.generators = {
-- 	12.0,
-- 	7.03448,
-- }

tuning.circle_of_fifths = { "F", "C", "G", "D", "A", "E", "B" }

tuning.octave = { 1 }
tuning.tone = { -1, 2 } -- whole tone
tuning.semitone = { 3, -5 } -- diatonic semitone
tuning.chroma = { -4, 7 } -- apotome, chromatic semitone
tuning.comma = { 7, -12 } -- Pythagorean comma / diesis

if tuning.rank > 2 then
	tuning.comma = { 0, 0, 1 }
end

tuning.snap_labels = { "Diatonic", "Chromatic", "Fine" }

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

function tuning.load()
	tuning.diatonic_table = tuning.generate_scale(7, 1)
	tuning.chromatic_table = tuning.generate_scale(12, 4)
	tuning.fine_table = tuning.generate_scale(31, 12)
	-- tuning.fine_table = tuning.generate_scale(19, 7)

	tuning.tables = { tuning.diatonic_table, tuning.chromatic_table, tuning.fine_table }
end

function tuning.new_interval()
	local new = {}
	for i = 1, tuning.rank do
		new[i] = 0
	end
	return new
end

-- Given some pitch p, find interval in current grid that is closest
-- TODO: rounding assumes scales is relatively even
function tuning.snap(p)
	local t = tuning.tables[project.settings.snap_pitch]
	assert(t)
	local steps = math.floor(p * (#t / 12) + 0.5)
	return tuning.from_table(t, steps)
end

-- Look up interval in table, correcting for octave offsets
function tuning.from_table(t, i)
	local s = #t
	local oct = math.floor(i / s)
	i = i - oct * s
	local p = t[i + 1]

	local new = tuning.new_interval()
	new[1] = p[1] + oct
	new[2] = p[2]

	return new
end

-- Indexed by midi number, middle C = midi note number 60.
-- Note: we currently assume #chromatic_table = 12 so this works.
function tuning.from_midi(n)
	return tuning.from_table(tuning.chromatic_table, n - 60)
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
