local device_list = require("device_list")
local log = require("log")

local function load_default_project()
	log.info("Loading default project")
	command.NewEffect.new(1, device_list.effects.limiter):run()

	local options = {
		name = "Epiano",
		instrument = device_list.instruments.epiano,
	}

	command.NewChannel.new(options):run()
	project.channels[2].armed = true
end

return load_default_project
