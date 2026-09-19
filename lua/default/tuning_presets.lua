local presets = {}

presets.tunings = {}

-- Order in the settings menu.
presets.list = {
	"meantone",
	"flattone",
	"archytas",
	"et_12",
	"et_15",
	"et_17",
	"et_19",
	"et_22",
	"et_24",
	"et_31",
	"et_34",
	"et_36",
	"et_41",
	"et_53",
	"et_72",
	"pythagorean",
	"ji_5",
	"ji_7",
	"ji_11",
	"septal",
}

-- temperaments

presets.tunings.meantone = {
	name = "Meantone",
	subgroup = "2.3.5.7",
	commas = { "81/80", "126/125" },
}

presets.tunings.flattone = {
	name = "Flattone",
	subgroup = "2.3.5.11",
	commas = { "81/80", "45/44" },
}

presets.tunings.archytas = {
	name = "Archytas",
	subgroup = "2.3.7",
	commas = { "64/63" },
}

-- equal temperaments

local function et(n, subgroup)
	return { name = n .. " equal", subgroup = subgroup, et = n }
end

presets.tunings.et_12 = et(12, "2.3.5")
presets.tunings.et_15 = et(15, "2.3.5")
presets.tunings.et_17 = et(17, "2.3.5")
presets.tunings.et_19 = et(19, "2.3.5.7")
presets.tunings.et_22 = et(22, "2.3.5.7")
presets.tunings.et_24 = et(24, "2.3.5.11")
presets.tunings.et_31 = et(31, "2.3.5.7")
presets.tunings.et_34 = et(34, "2.3.5")
presets.tunings.et_36 = et(36, "2.3.7")
presets.tunings.et_41 = et(41, "2.3.5.7.11")
presets.tunings.et_53 = et(53, "2.3.5")
presets.tunings.et_72 = et(72, "2.3.5.7.11")

-- just intonation

presets.tunings.pythagorean = { name = "Pythagorean", subgroup = "2.3" }
presets.tunings.ji_5 = { name = "5-limit JI", subgroup = "2.3.5" }
presets.tunings.ji_7 = { name = "7-limit JI", subgroup = "2.3.5.7" }
presets.tunings.ji_11 = { name = "11-limit JI", subgroup = "2.3.5.7.11" }
presets.tunings.septal = { name = "2.3.7 JI", subgroup = "2.3.7" }

-- TODO: default recommended is ups/downs
function presets.default()
	local def = util.clone(presets.tunings.meantone)
	def.n_accidentals = 0
	return def
end

return presets
