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


pitch = {}

function ratioToPitch(r)
	return 12.0 * math.log(r) / math.log(2)
end

-- default 11-limit JI table
gen_table = {
	ratioToPitch(2/1),
	ratioToPitch(3/2),
	ratioToPitch(81/80),
	ratioToPitch(64/63),
	ratioToPitch(33/32),
}


-- meantone TE optimal
-- gen_table = {
-- 	12.01397,
-- 	6.97049,
-- }

pitch_table = {
	{1},
	{0,1},
	{0,0,1},
	{0,0,0,1},
	{0,0,0,0,1},
	{},
	{},
}

diatonic_names = {"C", "D", "E", "F", "G", "A", "B"}

diatonic_table = {
	{ 0, 0},
	{-1, 2},
	{-2, 4},
	{ 1,-1},
	{ 0, 1},
	{-1, 3},
	{-2, 5},
}


for i,v in ipairs(pitch_table) do
	print(i,v)
end

function pitch:new() 
	return {
		coord = {
			0, -- octave / period
			0, -- fifth  / gen
			0, -- syntonic / plus,minus
			0, -- septimal / L-shaped
			0, -- quarter  / halfsharp,halfflat
			0, -- ups,downs 
			0, -- arrows
			0, -- free / offset (cents)
		},
		pitch = 60,
		name = "C"
	}
end