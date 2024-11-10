local backend = require("backend")
local deviceList = require("device_list")
local widgets = require("ui/widgets")
local Device = require("device")
local MidiHandler = require("midi_handler")

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
	project.transport.playing = false

	return project
end

function build.project()
	for _, v in ipairs(project.channels) do
		build.channel(v)
	end

	assert(#ui_channels == #project.channels)

	if #project.channels > 0 then
		selection.channel_index = 1
		selection.device_index = 0
	end
end

function build.channel(channel, ch_index)
	ch_index = ch_index or #project.channels

	local options = deviceList.instruments[channel.instrument.name]
	assert(options)

	local channel_ui = { effects = {} }
	table.insert(ui_channels, ch_index, channel_ui)
	channel_ui.instrument = Device:new(channel.name, channel.instrument.state, options)
	channel_ui.widget = widgets.Channel:new()

	backend:insertChannel(ch_index, channel.instrument.name)

	for i, v in ipairs(channel.effects) do
		build.effect(ch_index, i, v)
	end

	channel_ui.midi_handler = MidiHandler:new(options.n_voices, ch_index)
end

function build.effect(ch_index, effect_index, effect)
	local options = deviceList.effects[effect.name]
	assert(options)

	local effect_ui = Device:new(effect.name, effect.state, options)
	table.insert(ui_channels[ch_index].effects, effect_ui)

	backend:insertEffect(ch_index, effect_index, effect.name)
end

return build
