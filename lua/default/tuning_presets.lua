local presets = {}

-- equal temperaments

presets.et_19 = {
	-- 19-et meantone
	generators = {
		12.0,
		11 * (12 / 19),
	},
	type = "meantone",
	name = "19 Equal Temperament",
	fine = { 19, 8 },
}

presets.et_31 = {
	-- 31-et meantone
	-- Fifth = 18 steps
	-- 7/4 = A#
	-- 11/8 = Ex or Gbb
	generators = {
		12.0,
		18 * (12 / 31),
	},
	type = "meantone",
	name = "31 Equal Temperament",
}

presets.et_34 = {
	generators = {
		12,
		20 * (12 / 34),
		1 * (12 / 34),
	},
	type = "ji_5",
	name = "34 Equal Temperament",
	fine = "ji_5_34",
}

presets.et_41 = {
	generators = {
		12,
		24 * (12 / 41),
		1 * (12 / 41),
	},
	type = "ji_5",
	name = "41 Equal Temperament",
	fine = { 41 },
}

-- fifth based system

presets.meantone = {
	-- septimal meantone WE
	-- 7/4 = A#
	-- 11/8 = Ex or Gbb
	generators = {
		12.01236,
		6.97212,
	},
	type = "meantone",
	name = "Meantone",
}

presets.meantone_quarter = {
	generators = {
		12.0,
		6.96578,
	},
	type = "meantone",
	name = "1/4-comma Meantone",
}

presets.flattone = {
	-- flattone
	-- 7/4 = Bbb
	-- 11/8 = F#
	generators = {
		12.02062,
		6.94545,
	},
	type = "meantone",
	name = "Flattone",
}

presets.mavila = {
	-- equal beating
	generators = {
		12.00,
		6.76337,
	},
	type = "mavila",
	name = "Mavila",
	fine = { 16 },
}

presets.archytas = {
	-- Archytas / super-pythagorean
	-- 5/4 = D#
	-- 7/4 = Bb
	generators = {
		11.96955,
		7.07522,
	},
	type = "pyth",
	name = "Archytas",
}

-- rank 2 temperaments (~ 5-limit)

presets.porcupine = {
	generators = {
		12.00329,
		7.0796,
		0.51468,
	},
	type = "ji_5",
	name = "Porcupine",
}

presets.diaschismic = {
	-- Diaschismic with pure octaves
	generators = {
		12.00,
		7.04958,
		0.29748,
	},
	type = "ji_5",
	name = "Diaschismic",
}

-- rank 2 systems on other subgroups

-- TODO: fix fine scale
presets.semaphore = {
	generators = {
		12.00,
		7.01378,
		0,
		0.46555,
	},
	type = "septal",
	name = "Semaphore",
}

presets.slendric = {
	-- c = o - 5*f
	generators = {
		12.00486,
		7.01346,
		0,
		0.31576,
	},
	type = "septal",
	name = "Slendric",
}

-- rank 3 systems

presets.marvel = {
	-- Marvel (7-limit)
	-- 5/4 = E-
	-- 7/4 = A#--
	generators = {
		12.00597,
		7.00756,
		0.18001,
	},
	type = "ji_5",
	name = "Marvel (7-limit)",
}

presets.pele_7 = {
	-- Argent / Pele / Hemifamity (7-limit)
	-- 5/4 = E-
	-- 7/4 = Bb-
	generators = {
		11.9972,
		7.02664,
		0.24493,
	},
	type = "ji_5",
	name = "Pele (7-limit)",
}

presets.pele_11 = {
	-- Pele (11-limit)
	-- 5/4 = E-
	-- 7/4 = Bb-
	-- 11/8 = Gb-
	generators = {
		11.99542,
		7.03011,
		0.25316,
	},
	type = "ji_5",
	name = "Pele (11-limit)",
}

presets.akea = {
	-- Akea (11-limit)
	-- 5/4 = E-
	-- 7/4 = Bb-
	-- 11/8 = F++
	generators = {
		12.0014,
		7.02924,
		0.26236,
	},
	type = "ji_5",
	name = "Akea (11-limit)",
}

-- JI

presets.pythagorean = {
	generators = {
		"2/1",
		"3/2",
	},
	type = "pyth",
	name = "Pythagorean tuning",
}

presets.ji_5 = {
	generators = {
		"2/1",
		"3/2",
		"81/80",
	},
	type = "ji_5",
	name = "5-limit JI",
}

presets.septal = {
	generators = {
		"2/1",
		"3/2",
		0,
		"64/63",
	},
	type = "septal",
	name = "2.3.7 JI",
}

presets.ji_7 = {
	generators = {
		"2/1",
		"3/2",
		"81/80",
		"64/63",
	},
	type = "ji_7",
	name = "7-limit JI",
}

presets.ji_11 = {
	generators = {
		"2/1",
		"3/2",
		"81/80",
		"64/63",
		"33/32",
	},
	type = "ji_11",
	name = "11-limit JI",
}

-- scales

presets.scales = {}

-- 5-limit diatonic
presets.scales.zarlino = {
	"9/8",
	"5/4",
	"4/3",
	"3/2",
	"5/3",
	"15/8",
	"2/1",
}

-- 5-limit chromatic
presets.scales.duodene = {
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
presets.scales.ji_5_22 = {
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

-- 5-limit scale for 34et
-- doubled 17et chain of fifths with 81/80 offsets
presets.scales.ji_5_34 = {
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
presets.scales.septal_7 = {
	"9/8",
	"9/7",
	"4/3",
	"3/2",
	"12/7",
	"27/14",
	"2/1",
}

-- 2.3.7 chromatic
presets.scales.septal_12 = {
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
presets.scales.septal_36 = {
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
presets.scales.mavila_12 = {
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

return presets
