local presets = {}

-- rank 2 systems

presets.pythagorean = {
	generators = {
		"2/1",
		"3/2",
	},
	type = "pyth",
	name = "Pythagorean tuning",
}

presets.meantone = {
	-- septimal meantone WE
	-- 7/4 = A#
	-- 11/8 = Ex or Gbb
	generators = {
		12.01236,
		6.97212,
	},
	type = "meantone",
	name = "Septimal meantone",
}

presets.meantone_quarter = {
	generators = {
		12.01236,
		6.97212,
	},
	type = "meantone",
	name = "1/4-comma meantone",
}

presets.meantone_31et = {
	-- 31-et meantone
	-- Fifth = 18 steps
	-- 7/4 = A#
	-- 11/8 = Ex or Gbb
	generators = {
		12.0,
		18 * (12 / 31),
	},
	type = "meantone",
	name = "31-ET meantone",
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

presets.archytas = {
	-- Archytas / super-pythagorean
	-- 5/4 = D#
	-- 7/4 = Bb
	generators = {
		11.96955,
		7.07522,
	},
	type = "pyth",
	name = "Archytas/superpyth",
}

presets.diaschismic = {
	-- Diaschismic with pure octaves
	generators = {
		6.00,
		1.04958,
	},
	type = "diaschismic",
	name = "Diaschismic",
}

-- rank 3 systems

presets.ji_5 = {
	generators = {
		"2/1",
		"3/2",
		"81/80",
	},
	type = "ji_5",
	name = "5-limit JI",
}

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
	name = "Marvel temperament (7-limit)",
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
	name = "Pele temperament (7-limit)",
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
	name = "Pele temperament (11-limit)",
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
	name = "Akea temperament (11-limit)",
}

presets.et_41 = {
	-- 41-et rank3
	-- fifth = 24 steps
	-- accidental = 1 step
	generators = {
		12,
		24 * (12 / 41),
		1 * (12 / 41),
	},
	type = "ji_5",
	name = "41-ET, rank-3 structure",
}

-- higher rank (currently not working!)

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

presets.scales.zarlino = {
	"9/8",
	"5/4",
	"4/3",
	"3/2",
	"5/3",
	"15/8",
	"2/1",
}

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

presets.scales.ji_5_fine = {
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

return presets
