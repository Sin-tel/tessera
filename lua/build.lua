local Channel = require("channel")
local Device = require("device")
local device_list = require("device_list")
local engine = require("engine")
local widgets = require("ui/widgets")

local build = {}

-- clear the project from a valid state
function build.new_project()
	-- make sure there is no lingering state on the backend.
	engine.stop()
	tessera.audio.clear_messages()

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
	project.transport.start_time = 0
	project.transport.recording = true
end

-- build the given "project" is set but nothing else is
local function setup_project()
	for i, v in ipairs(project.channels) do
		build.channel(i, v)
	end

	assert(#ui_channels == #project.channels)

	if #project.channels > 0 then
		selection.ch_index = 1
		selection.device_index = 0
	end

	engine.seek(project.transport.start_time)
end

-- restore project from invalid state where backend died
function build.restore_project()
	ui_channels = {}
	setup_project()
end

-- load new project from a valid state
function build.load_project(new)
	build.new_project()
	project = new
	setup_project()
end

function build.channel(ch_index, channel_data)
	local options = device_list.instruments[channel_data.instrument.name]

	assert(options, 'Could not find options for "' .. channel_data.instrument.name .. '"')
	local meter_id = tessera.audio.insert_channel(ch_index, channel_data.instrument.name)

	local instrument = Device.new(channel_data.instrument, options, meter_id)
	local widget = widgets.Channel.new()
	local channel = Channel.new(ch_index, channel_data, instrument, widget)
	table.insert(ui_channels, ch_index, channel)

	for i, v in ipairs(channel_data.effects) do
		build.effect(ch_index, i, v)
	end

	build.refresh_channels()
end

function build.effect(ch_index, effect_index, effect)
	local options = device_list.effects[effect.name]
	assert(options)

	local meter_id = tessera.audio.insert_effect(ch_index, effect_index, effect.name)

	local effect_ui = Device.new(effect, options, meter_id)
	table.insert(ui_channels[ch_index].effects, effect_index, effect_ui)
end

function build.refresh_channels()
	for i, v in ipairs(ui_channels) do
		v.ch_index = i
		v.roll.ch_index = i
	end
end

return build
