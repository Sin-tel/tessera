--[[
pitch system is based on a diatonic (octave/fifth or period/gen) generated scale (can be any rank 2 MOS)
which is notated with alphabetic characters + b/#
there is also 6 extra accidental pairs
  -> total max 8 dimensions (e.g. up to 19-limit JI)
(+ free offset in cents)

base pitch is always 60 = C4 = 261.63Hz

there are seperate accidentals for "double half sharp/flat"
when 243/242 is tempered out (rastmic), these should revert to normal sharps/flats
(this is true for 17, 24, 31, 41, 72)

4 layers:
  note names  (only view)
    | convert diatonic scale notation to oct/fifth pair
  notation coordinates (internal representation)
    | look up generator mapping
  generator coordinates
    | look up generator sizes
  pitch in semitones/midi number

( possible modal + domal? -> eg 3*4 chromatic grid moving by fifths and thirds )
(  --> specify moves e.g. keys d and c move by third instead of whole syntonic )

( also possible: just make the home row a fixed arbitrary scale eg overtone scale)

info for tuning specification:
  - size of all generators (first two are period and gen)
  - which accidentals are used
     - mapping of these to generators
  - root of diatonic (default C)
  - size of diatonic scale + how many gens down
  - (optional chromatic scale?)

home QWERTY row -> current diatonic
system always remembers accidentals last used for each note in the diatonic

(if chromatic specified -- use q2w3e4r5t6y7u8i9o0p)?
az - octave up/down        shift+ ~ move selection updown
sx - sharp up / flat down (it actually moves the scale by a fifth but the root stays on C so it adds a sharp..)
                          ( w shift it does move a chromatic semitone)
dc - plus up / minus down
...

tracker-like keyboard nav!

finetuning w mouse--
]]

local Pitch = {}

-- default 11-limit JI table
-- Pitch.generators = {
-- 	ratio(2/1),
-- 	ratio(3/2),
-- 	ratio(81/80),
-- 	ratio(64/63),
-- 	ratio(33/32),
-- }

-- -- meantone TE optimal
-- Pitch.generators = {
-- 	12.01397,
-- 	6.97049,
-- }

-- meantone target tuning (5/4, 2)
Pitch.generators = {
	12.0,
	6.96578,
}

-- temperament projection matrix
-- empty entries are zero
-- stylua: ignore
Pitch.table = {
	{ 1 },
	{ 0, 1 },
	{},
	{},
	{},
	{},
	{},
}

Pitch.diatonic_names = { "C", "D", "E", "F", "G", "A", "B" }

-- current scale expressed in generator steps
-- stylua: ignore
Pitch.diatonic_table = {
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
Pitch.chromatic_table = {
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

-- for i, v in ipairs(Pitch.table) do
-- print(i, v)
-- end

function Pitch:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.coord = {
		0, -- octave / period
		0, -- fifth  / gen
		0, -- syntonic / plus,minus
		0, -- septimal / L-shaped
		0, -- quarter  / halfsharp,halfflat
		0, -- ups,downs
		0, -- arrows
		0, -- free / offset (cents)
	}
	new.pitch = 60
	-- new.name = "C"
	return new
end

-- diatonic home row, 1 = C5
function Pitch:fromDiatonic(n, add_octave)
	local add_octave = add_octave or 0
	local new = Pitch:new()
	local s = #Pitch.diatonic_table
	local oct = math.floor((n - 1) / s)
	n = n - oct * s
	local dia = Pitch.diatonic_table[n]
	new.coord[1] = dia[1] + oct + add_octave
	new.coord[2] = dia[2]
	new:recalc()

	return new
end

-- indexed by midi number, C5 = 72
function Pitch:fromMidi(n)
	local new = Pitch:new()
	local s = #Pitch.chromatic_table
	local oct = math.floor(n / s)
	n = n - oct * s
	local dia = Pitch.chromatic_table[n + 1]
	new.coord[1] = dia[1] + oct - 6
	new.coord[2] = dia[2]

	new:recalc()

	return new
end

function Pitch:recalc()
	local f = 72
	for i, v in ipairs(self.coord) do
		f = f + v * (self.generators[i] or 0)
	end
	self.pitch = f
end

return Pitch
