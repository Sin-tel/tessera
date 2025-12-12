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

-- -- meantone TE optimal
tuning.rank = 2
tuning.generators = {
	12.01397,
	6.97049,
}

-- flattone
-- tuning.rank = 2
-- tuning.generators = {
-- 	12.02062,
-- 	6.94545,
-- }

-- archytas
-- tuning.generators = {
-- 	11.96955,
-- 	7.07522,
-- }

-- 29edo
-- tuning.generators = {
-- 	12.0,
-- 	7.03448,
-- }

-- septimal meantone WE
-- tuning.generators = {
-- 	12.01236,
-- 	6.97212,
-- }

-- -- meantone target tuning (5/4, 2)
-- tuning.generators = {
-- 	12.0,
-- 	6.96578,
-- }

-- -- meantone target tuning (5/4, 6/5)
-- tuning.generators = {
-- 	12.10753,
-- 	7.01955,
-- }

tuning.circle_of_fifths = { "F", "C", "G", "D", "A", "E", "B" }

-- current scale expressed in generator steps
-- stylua: ignore
tuning.diatonic_table = {
	{  0,  0 },
	{ -1,  2 },
	{ -2,  4 },
	{  1, -1 },
	{  0,  1 },
	{ -1,  3 },
	{ -2,  5 },
}

-- scale used for 12edo input (e.g. midi keyboard)
-- stylua: ignore
tuning.chromatic_table = {
	{  0,  0 },  -- C
	{ -4,  7 },  -- C#
	{ -1,  2 },  -- D
	{  2, -3 },  -- Eb
	{ -2,  4 },  -- E
	{  1, -1 },  -- F
	{ -3,  6 },  -- F#
	{  0,  1 },  -- G
	{  3, -4 },  -- Ab
	{ -1,  3 },  -- A
	{  2, -2 },  -- Bb
	{ -2,  5 },  -- B
}

tuning.octave = { 1 }
tuning.tone = { -1, 2 } -- whole tone
tuning.semitone = { 3, -5 } -- diatonic semitone
tuning.chroma = { -4, 7 } -- apotome, chromatic semitone
tuning.comma = { 7, -12 } -- Pythagorean comma / diesis

if tuning.rank > 2 then
	tuning.comma = { 0, 0, 1 }
end

function tuning.new_note()
	local new = {}
	for i = 1, tuning.rank do
		new[i] = 0
	end
	return new
end

-- diatonic home row, 1 = C5
function tuning.from_diatonic(n, add_octave)
	add_octave = add_octave or 0
	local s = #tuning.diatonic_table
	local oct = math.floor((n - 1) / s)
	n = n - oct * s
	local dia = tuning.diatonic_table[n]

	local new = tuning.new_note()
	new[1] = dia[1] + oct + add_octave
	new[2] = dia[2]

	return new
end

-- indexed by midi number, middle C = midi note number 60
function tuning.from_midi(n)
	local s = #tuning.chromatic_table
	local oct = math.floor(n / s)
	n = n - oct * s
	local dia = tuning.chromatic_table[n + 1]

	local new = tuning.new_note()
	new[1] = dia[1] + oct - 5
	new[2] = dia[2]

	return new
end

-- coordinates to pitch
function tuning.get_pitch(p)
	local f = 60
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
		elseif plus < 0 then
			local minus = -plus
			acc = acc .. string.rep("m", minus)
		end
	end

	local n = tuning.circle_of_fifths[n_i % #tuning.circle_of_fifths + 1]

	return n .. acc .. tostring(o)
end

function tuning.get_diatonic_index(p)
	return 1 + 7 * p[1] + 4 * p[2]
end

function tuning.get_chromatic_index(p)
	return 1 + 12 * p[1] + 7 * p[2]
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

return tuning
