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

tuning.diatonic_names = { "C", "D", "E", "F", "G", "A", "B" }

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
function tuning:fromDiatonic(n, add_octave)
	add_octave = add_octave or 0
	local s = #tuning.diatonic_table
	local oct = math.floor((n - 1) / s)
	n = n - oct * s
	local dia = tuning.diatonic_table[n]

	local new = {}
	new[1] = dia[1] + oct + add_octave
	new[2] = dia[2]

	return self:getPitch(new)
end

-- indexed by midi number, middle C = midi note number 60
function tuning:fromMidi(n)
	local s = #tuning.chromatic_table
	local oct = math.floor(n / s)
	n = n - oct * s
	local dia = tuning.chromatic_table[n + 1]

	local new = {}
	new[1] = dia[1] + oct - 5
	new[2] = dia[2]

	return self:getPitch(new)
end

-- coordinates to pitch
function tuning:getPitch(p)
	local f = 60
	-- we dont use ipairs because entries may be nil, which is implicitly zero
	for i, v in pairs(p) do
		f = f + v * (self.generators[i] or 0)
	end
	return f
end

return tuning
