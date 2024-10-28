local serialize = require("lib/serialize")
local log = require("log")
local save = {}

function save.write(filename)
	log.info('saving project "' .. filename .. '"')

	local content = serialize(project, "project")
	util.writefile(filename, content)
end

function save.read(filename)
	log.info('loading project "' .. filename .. '"')

	local content = util.readfile(filename)
	project = setfenv(loadstring(content), {})()
	for _, v in ipairs(project.channels) do
		channelHandler.buildChannel(v)
	end

	assert(#ui_channels == #project.channels)

	if #project.channels > 0 then
		selection.channel_index = 1
		selection.device_index = 0
	end
end

return save
