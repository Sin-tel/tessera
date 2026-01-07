local log = require("log")

local function load_default_project()
	log.info("Loading default project")
	command.NewEffect.new(1, "limiter"):run()
	command.NewChannel.new("epiano"):run()
	project.channels[2].armed = true
end

return load_default_project
