local Device = require("device")
local Roll = require("roll")
local VoiceAlloc = require("voice_alloc")
local device_list = require("device_list")
local widgets = require("ui/widgets")

local build = {}

function build.new_project()
	if project.channels then
		for i = #project.channels, 1, -1 do
			tessera.audio.remove_channel(i)
		end
	end

	ui_channels = {}
	-- clear selection
	selection.ch_index = nil
	selection.device_index = nil

	-- init empty project
	project = {}
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
end

function build.project()
	for i, v in ipairs(project.channels) do
		build.channel(i, v)
	end

	assert(#ui_channels == #project.channels)

	if #project.channels > 0 then
		selection.ch_index = 1
		selection.device_index = 0
	end
end

function build.channel(ch_index, channel)
	local options = device_list.instruments[channel.instrument.name]
	assert(options)

	local channel_ui = { effects = {} }
	table.insert(ui_channels, ch_index, channel_ui)
	channel_ui.instrument = Device.new(channel.name, channel.instrument.state, options)
	channel_ui.widget = widgets.Channel.new()

	tessera.audio.insert_channel(ch_index, channel.instrument.name)

	for i, v in ipairs(channel.effects) do
		build.effect(ch_index, i, v)
	end

	channel_ui.voice_alloc = VoiceAlloc.new(ch_index, options.n_voices)
	channel_ui.roll = Roll.new(ch_index)

	build.refresh_channels()
end

function build.effect(ch_index, effect_index, effect)
	local options = device_list.effects[effect.name]
	assert(options)

	local effect_ui = Device.new(effect.name, effect.state, options)
	table.insert(ui_channels[ch_index].effects, effect_index, effect_ui)

	tessera.audio.insert_effect(ch_index, effect_index, effect.name)
end

function build.refresh_channels()
	for i, v in ipairs(ui_channels) do
		v.voice_alloc.ch_index = i
		v.roll.ch_index = i
	end
end

return build
