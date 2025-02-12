local backend = require("backend")
local deviceList = require("device_list")
local widgets = require("ui/widgets")
local Device = require("device")
local VoiceAlloc = require("voice_alloc")
local Roll = require("roll")

local build = {}

function build.newProject()
	-- init empty project
	local project = {}
	project.channels = {}
	project.VERSION = {}
	project.VERSION.MAJOR = VERSION.MAJOR
	project.VERSION.MINOR = VERSION.MINOR
	project.VERSION.PATCH = VERSION.PATCH
	project.name = "Untitled project"
	project.transport = {}
	project.transport.time = 0
	project.transport.start_time = 0
	project.transport.recording = true

	return project
end

function build.project()
	for _, v in ipairs(project.channels) do
		build.channel(v)
	end

	assert(#ui_channels == #project.channels)

	if #project.channels > 0 then
		selection.ch_index = 1
		selection.device_index = 0
	end
end

function build.channel(channel)
	local ch_index = #ui_channels + 1

	local options = deviceList.instruments[channel.instrument.name]
	assert(options)

	local channel_ui = { effects = {} }
	table.insert(ui_channels, ch_index, channel_ui)
	channel_ui.instrument = Device.new(channel.name, channel.instrument.state, options)
	channel_ui.widget = widgets.Channel.new()

	backend:insertChannel(ch_index, channel.instrument.name)

	for i, v in ipairs(channel.effects) do
		build.effect(ch_index, i, v)
	end

	channel_ui.voice_alloc = VoiceAlloc.new(ch_index, options.n_voices)
	channel_ui.roll = Roll.new(ch_index)
end

function build.effect(ch_index, effect_index, effect)
	local options = deviceList.effects[effect.name]
	assert(options)

	local effect_ui = Device.new(effect.name, effect.state, options)
	table.insert(ui_channels[ch_index].effects, effect_ui)

	backend:insertEffect(ch_index, effect_index, effect.name)
end

return build
