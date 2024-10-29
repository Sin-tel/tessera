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

local setup_path = love.filesystem.getSource() .. "/settings/setup.lua"

function save.writeSetup()
	local content = serialize(setup, "setup")
	util.writefile(setup_path, content)
end

function save.readSetup()
	local setup
	if util.fileExists(setup_path) then
		local content = util.readfile(setup_path)
		setup = setfenv(loadstring(content), {})()
	else
		setup = {}
		setup.audio = {}
		setup.audio.default_host = "default"
		setup.audio.default_device = "default"
		setup.audio.buffer_size = 128
		setup.midi = {}
		setup.midi.inputs = { { name = "default" } }

		save.writeSetup()
	end

	return setup
end

return save
