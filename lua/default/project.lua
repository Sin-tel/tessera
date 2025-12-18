local log = require("log")

local function load_default_project()
	log.info("Loading default project")
	command.NewChannel.new("epiano"):run()
	project.channels[1].armed = true
end

return load_default_project
