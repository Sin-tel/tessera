--[[
pitch system is based on a diatonic (octave/fifth or period/gen) generated scale (can be any rank 2 MOS)
which is notated with alphabetic characters + b/#
there is also 6 extra accidental pairs
  -> total max 8 dimensions (e.g. up to 19-limit JI)
(+ free offset in cents)

base pitch is always 60 = C4 = 261.63Hz

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

Pitch = {}

-- default 11-limit JI table
-- gen_table = {
-- 	ratio(2/1),
-- 	ratio(3/2),
-- 	ratio(81/80),
-- 	ratio(64/63),
-- 	ratio(33/32),
-- }

-- meantone TE optimal
Pitch.generators = {
	12.01397,
	6.97049,
}

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

Pitch.diatonic_table = {
	{ 0, 0 },
	{ -1, 2 },
	{ -2, 4 },
	{ 1, -1 },
	{ 0, 1 },
	{ -1, 3 },
	{ -2, 5 },
}

for i, v in ipairs(Pitch.table) do
	print(i, v)
end

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
	new.name = "C"

	return new
end

function Pitch:newFromDiatonic(n)
	local new = Pitch:new()

	local s = #Pitch.diatonic_table
	print(n, s)
	local oct = math.floor((n - 1) / s)
	n = n - oct * s
	print(oct, n)
	local dia = Pitch.diatonic_table[n]
	new.coord[1] = dia[1] + oct
	new.coord[2] = dia[2]

	-- for i,v in ipairs(new.coord) do
	-- 	print(i,v)
	-- end

	new:recalc()

	return new
end

function Pitch:recalc()
	local f = 60
	for i, v in ipairs(self.coord) do
		f = f + v * (self.generators[i] or 0)
	end
	self.pitch = f
end
