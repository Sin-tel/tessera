local presets = {}

-- categories and order in the settings menu
presets.categories = {
	{
		name = "Temperament",
		list = {
			-- rank 2
			"meantone",
			"flattone",
			"tetracot",
			"kleismic",
			"porcupine",
			"diaschismic",
			"pajara",
			"archytas",
			"slendric",
			"mothra",
			"rodan",
			"miracle",
			-- rank 3
			"marvel",
			"pele",
			"akea",
		},
	},
	{
		-- TODO: 5 and 7 et kinda broken right now
		name = "Equal",
		list = {
			-- "et_5",
			-- "et_7",
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
		},
	},
	{
		name = "Just Intonation",
		list = { "pythagorean", "ji_5", "ji_7", "ji_11", "ji_2_3_7" },
	},
}

local tunings = {}

-- temperaments

tunings.meantone = {
	name = "Meantone",
	subgroup = "2.3.5.7",
	commas = { "81/80", "126/125" },
}

tunings.flattone = {
	name = "Flattone",
	subgroup = "2.3.5.11",
	commas = { "81/80", "45/44" },
}

tunings.porcupine = {
	name = "Porcupine",
	subgroup = "2.3.5.11",
	commas = { "55/54", "100/99" },
}

tunings.tetracot = {
	name = "Tetracot",
	subgroup = "2.3.5.11",
	commas = { "100/99", "243/242" },
}

-- note: add .13 once font supports it
tunings.kleismic = {
	name = "Kleismic",
	subgroup = "2.3.5",
	commas = { "15625/15552" },
}

tunings.diaschismic = {
	name = "Diaschismic",
	subgroup = "2.3.5.7",
	commas = { "2048/2025", "126/125" },
}

tunings.pajara = {
	name = "Pajara",
	subgroup = "2.3.5.7",
	commas = { "50/49", "64/63" },
}

tunings.archytas = {
	name = "Archytas",
	subgroup = "2.3.7",
	commas = { "64/63" },
}

tunings.slendric = {
	name = "Slendric",
	subgroup = "2.3.7",
	commas = { "1029/1024" },
}

-- slendric + meantone
tunings.mothra = {
	name = "Mothra",
	subgroup = "2.3.5.7",
	commas = { "1029/1024", "81/80" },
}

-- slendric + 5120/5103
tunings.rodan = {
	name = "Rodan",
	subgroup = "2.3.5.7",
	commas = { "1029/1024", "245/243" },
}

tunings.miracle = {
	name = "Miracle",
	subgroup = "2.3.5.7.11",
	commas = { "225/224", "385/384", "441/440" },
}

--- rank 3

tunings.marvel = {
	name = "Marvel",
	subgroup = "2.3.5.7.11",
	commas = { "225/224", "385/384" },
}

tunings.pele = {
	name = "Pele",
	subgroup = "2.3.5.7.11",
	commas = { "441/440", "896/891" },
}

tunings.akea = {
	name = "Akea",
	subgroup = "2.3.5.7.11",
	commas = { "385/384", "2200/2187" },
}

-- equal temperaments

local function et(n, subgroup)
	return { name = n .. " equal", subgroup = subgroup, et = n }
end

-- tunings.et_5 = et(5, "2.3.5.7")
-- tunings.et_7 = et(7, "2.3.5.7")
tunings.et_12 = et(12, "2.3.5")
tunings.et_15 = et(15, "2.3.5")
tunings.et_17 = et(17, "2.3.7")
tunings.et_19 = et(19, "2.3.5.7")
tunings.et_22 = et(22, "2.3.5.7")
tunings.et_24 = et(24, "2.3.5.11")
tunings.et_31 = et(31, "2.3.5.7")
tunings.et_34 = et(34, "2.3.5")
tunings.et_36 = et(36, "2.3.7")
tunings.et_41 = et(41, "2.3.5.7.11")
tunings.et_53 = et(53, "2.3.5")
tunings.et_72 = et(72, "2.3.5.7.11")

-- just intonation

tunings.pythagorean = { name = "Pythagorean", subgroup = "2.3" }
tunings.ji_5 = { name = "5-limit JI", subgroup = "2.3.5" }
tunings.ji_7 = { name = "7-limit JI", subgroup = "2.3.5.7" }
tunings.ji_11 = { name = "11-limit JI", subgroup = "2.3.5.7.11" }
tunings.ji_2_3_7 = { name = "2.3.7 JI", subgroup = "2.3.7" }

-- Index into presets.categories that a definition belongs to.
-- Works on any definition, not just the presets.
function presets.category(def)
	if def.et then
		return 2
	elseif def.commas and #def.commas > 0 then
		return 1
	else
		return 3
	end
end

-- Key of the preset with the same name as def, if there is one.
function presets.find(def)
	for k, v in pairs(tunings) do
		if v.name == def.name then
			return k
		end
	end
end

-- Meantone with plain sharps and flats.
-- Not the recommended notation (ups and downs) on purpose: new users should be able to read it.
function presets.default()
	local def = util.clone(tunings.meantone)
	def.notation = { accidentals = 0, half_sharp = false }
	return def
end

-- Every preset is listed exactly once, in the category it belongs to.
if debug then
	local listed = {}
	for i, c in ipairs(presets.categories) do
		for _, k in ipairs(c.list) do
			local def = tunings[k]
			assert(def, "Unknown preset in " .. c.name .. ": " .. k)
			assert(not listed[k], "Preset listed twice: " .. k)
			assert(presets.category(def) == i, "Preset in the wrong category: " .. k)
			listed[k] = true
		end
	end
	for k in pairs(tunings) do
		assert(listed[k], "Preset not in any category: " .. k)
	end
end

presets.tunings = tunings

return presets
