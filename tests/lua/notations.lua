-- print the notations for all preset tunings
-- cargo run -- --test-run tests/lua/notations.lua

local ProjectSettings = require("views/project_settings")
local presets = require("default.tuning_presets")
local tuning = require("tuning")

project = { settings = { snap_pitch = 1 }, channels = {} }
tuning.load_project()

local view = ProjectSettings.new()
view:sync()
assert(view.option, "no notation selected")
assert(project.settings.tuning.notation.accidentals == 0, "default is not plain")

for category, list in ipairs(presets.categories) do
	for _, key in ipairs(list.list) do
		view.tuning_mode = category
		view:set_category(category)
		view:select_preset(key)
		assert(presets.tunings[key].notation == nil, "preset was modified: " .. key)

		for _, row in ipairs(view.rows) do
			view:select_notation(row.option)
			assert(tuning.set(view.def), key)

			view:sync()
			assert(view:preset_keys()[view.preset_index] == key, "wrong preset after loading " .. key)
			assert(view.preset_dropdown.title == nil, "preset not found after loading " .. key)
			assert(row:is_current(), "wrong notation after loading " .. key)

			local map = tuning.input_map()
			-- one octave of keys, the octave above the root included
			assert(#map == #tuning.chromatic + 1, "input map size for " .. key)
			for i, entry in ipairs(map) do
				assert(entry.name ~= "", "no name for midi key " .. i .. " in " .. key)
			end

			local primes = {}
			for _, p in ipairs(row.primes) do
				table.insert(primes, p.ratio .. ": " .. p.name)
			end
			print(
				string.format(
					"%-12s %-14s %s %s",
					key,
					row.title,
					row.option.recommended and "->" or "  ",
					table.concat(primes, "  ")
				)
			)
		end
	end
end
