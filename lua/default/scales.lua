-- Just intonation scales, as ratios above the unison up to the octave.

local scales = {}

-- 5-limit diatonic
scales.zarlino = {
	"9/8",
	"5/4",
	"4/3",
	"3/2",
	"5/3",
	"15/8",
	"2/1",
}

-- 5-limit chromatic
scales.duodene = {
	"16/15",
	"9/8",
	"6/5",
	"5/4",
	"4/3",
	"45/32",
	"3/2",
	"8/5",
	"5/3",
	"9/5",
	"15/8",
	"2/1",
}

-- 5-limit 22-note scale
scales.ji_5_22 = {
	"25/24",
	"16/15",
	"10/9",
	"9/8",
	"32/27",
	"6/5",
	"5/4",
	"81/64",
	"4/3",
	"25/18",
	"45/32",
	"40/27",
	"3/2",
	"25/16",
	"8/5",
	"5/3",
	"27/16",
	"16/9",
	"9/5",
	"15/8",
	"243/128",
	"2/1",
}

-- duodene + 10/9, 27/20, 16/9
scales.fine_15 = {
	"16/15",
	"10/9",
	"9/8",
	"6/5",
	"5/4",
	"4/3",
	"27/20",
	"45/32",
	"3/2",
	"8/5",
	"5/3",
	"16/9",
	"9/5",
	"15/8",
	"2/1",
}

-- 5-limit scale for 34et
-- doubled 17et chain of fifths with 81/80 offsets
scales.ji_5_34 = {
	"81/80",
	"256/243",
	"16/15",
	"2187/2048",
	"10/9",
	"9/8",
	"729/640",
	"32/27",
	"6/5",
	"19683/16384",
	"5/4",
	"81/64",
	"320/243",
	"4/3",
	"27/20",
	"1024/729",
	"45/32",
	"729/512",
	"40/27",
	"3/2",
	"243/160",
	"128/81",
	"8/5",
	"6561/4096",
	"5/3",
	"27/16",
	"1280/729",
	"16/9",
	"9/5",
	"59049/32768",
	"15/8",
	"243/128",
	"160/81",
	"2/1",
}

-- 2.3.7 diatonic
scales.septal_7 = {
	"9/8",
	"9/7",
	"4/3",
	"3/2",
	"12/7",
	"27/14",
	"2/1",
}

-- 2.3.7 chromatic
scales.septal_12 = {
	"28/27",
	"9/8",
	"7/6",
	"9/7",
	"4/3",
	"81/56",
	"3/2",
	"14/9",
	"12/7",
	"7/4",
	"27/14",
	"2/1",
}

-- 2.3.7 36et scale
scales.septal_36 = {
	"49/48",
	"28/27",
	"256/243",
	"243/224",
	"54/49",
	"9/8",
	"8/7",
	"7/6",
	"32/27",
	"98/81",
	"243/196",
	"81/64",
	"9/7",
	"21/16",
	"4/3",
	"49/36",
	"112/81",
	"729/512",
	"81/56",
	"189/128",
	"3/2",
	"32/21",
	"14/9",
	"128/81",
	"729/448",
	"81/49",
	"27/16",
	"12/7",
	"7/4",
	"16/9",
	"49/27",
	"729/392",
	"243/128",
	"27/14",
	"63/32",
	"2/1",
}

-- Mavila fifth-chain (has negative steps when tempered)
scales.mavila_12 = {
	"2187/2048",
	"9/8",
	"32/27",
	"81/64",
	"4/3",
	"729/512",
	"3/2",
	"128/81",
	"27/16",
	"16/9",
	"243/128",
	"2/1",
}

-- Scales to try for each snap setting, in order of preference.
-- The first one that works in the tuning is used, see TuningSystem::new.
-- If none work, it falls back on a chain of fifths.
scales.candidates = {
	diatonic = { scales.zarlino, scales.septal_7 },
	chromatic = { scales.duodene, scales.septal_12 },
	fine = { scales.ji_5_22, scales.septal_36 },
}

return scales
