--[[
pitch system is based on a diatonic (octave/fifth or period/gen) generated scale (can be any rank 2 MOS)
which is notated with alphabetic characters + b/#
there is also 6 extra accidental pairs
  -> total max 8 dimensions (e.g. up to 19-limit JI)
]]

-- TODO: load tuning info from file

local tuning = {}

-- default 11-limit JI table
-- tuning.generators = {
-- 	ratio(2/1),
-- 	ratio(3/2),
-- 	ratio(81/80),
-- 	ratio(64/63),
-- 	ratio(33/32),
-- }

-- meantone TE optimal
tuning.generators = {
	12.01397,
	6.97049,
}

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

-- temperament projection matrix
-- empty entries are zero
-- stylua: ignore
tuning.table = {
	{ 1 },
	{ 0, 1 },
	{},
	{},
	{},
	{},
	{},
}

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

-- diatonic home row, 1 = C5
function tuning.fromDiatonic(n, add_octave)
	add_octave = add_octave or 0
	local s = #tuning.diatonic_table
	local oct = math.floor((n - 1) / s)
	n = n - oct * s
	local dia = tuning.diatonic_table[n]

	local new = {}
	new[1] = dia[1] + oct + add_octave
	new[2] = dia[2]

	return tuning.getPitch(new)
end

-- indexed by midi number, middle C = midi note number 60
function tuning.fromMidi(n)
	local s = #tuning.chromatic_table
	local oct = math.floor(n / s)
	n = n - oct * s
	local dia = tuning.chromatic_table[n + 1]

	local new = {}
	new[1] = dia[1] + oct - 5
	new[2] = dia[2]

	-- return tuning.getPitch(new)
	return new
end

-- coordinates to pitch
function tuning.getPitch(p)
	local f = 60
	for i, v in ipairs(p) do
		f = f + v * (tuning.generators[i] or 0)
	end
	return f
end

-- TODO: generalize this to other systems
function tuning.getName(p)
	-- factor 4/7 is because base note name does not change when altering by an apotome (#) which is [-4, 7]
	local o = p[1] + math.floor(p[2] * 4 / 7) + 4
	local n_i = (p[2] + 1)
	local sharps = math.floor(n_i / #tuning.circle_of_fifths)

	local acc = ""
	if sharps > 0 then
		if sharps % 2 == 1 then
			acc = "t" -- #
		end
		local double_sharps = math.floor(sharps / 2)
		-- x
		acc = acc .. string.rep("y", double_sharps)
	elseif sharps < 0 then
		local flats = -sharps
		if flats == 1 then
			acc = "e" --b
		elseif flats == 2 then
			acc = "w" --bb
		elseif flats == 3 then
			acc = "q" --bbb
		else
			local group = (flats - 1) % 3
			if group == 0 then
				acc = "ww" -- bb bb
			elseif group == 1 then
				acc = "wq" -- bb bbb
			else
				acc = "qq" -- bbb bbb
			end
			local triple_flats = math.floor((flats - 4) / 3)
			if triple_flats > 0 then
				--- bbb
				acc = acc .. string.rep("q", triple_flats)
			end
		end
	end

	local n = tuning.circle_of_fifths[n_i % #tuning.circle_of_fifths + 1]

	return n .. acc .. tostring(o)
end

return tuning
