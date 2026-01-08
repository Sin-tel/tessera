local Channel = require("channel")
local Device = require("device")
local SliderValue = require("ui/slider_value")
local device_list = require("device_list")
local empty_project = require("default.empty_project")
local engine = require("engine")
local tuning = require("tuning")

local build = {}

local find_hue

-- build the given "project" is set but nothing else is
local function setup_project()
	for i, v in ipairs(project.channels) do
		build.channel(i, v)
	end

	assert(#ui_channels == #project.channels)

	if #project.channels > 0 then
		selection.select_default_channel()
		selection.device_index = nil
	end

	tuning.load(project.settings.tuning_key)
	engine.seek(project.transport.start_time)
end

-- clear the project from a valid state
function build.empty_project()
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
	project = empty_project()
end

-- restore project from invalid state where backend died
function build.restore_project()
	ui_channels = {}
	setup_project()
end

-- init new project from a valid state
function build.new_project()
	build.empty_project()
	-- add master channel
	project.channels[1] = build.new_channel_data({ master = true, name = "Master" })
	setup_project()
end

-- load project from a valid state
function build.load_project(new)
	build.empty_project()
	project = new
	setup_project()
end

function build.channel(ch_index, channel_data)
	local meter_id_channel = tessera.audio.insert_channel(ch_index)

	local instrument
	if channel_data.instrument then
		local options = device_list.instruments[channel_data.instrument.name]
		assert(options, 'Could not find options for "' .. channel_data.instrument.name .. '"')
		local meter_id_instrument = tessera.audio.insert_instrument(ch_index, channel_data.instrument.name)
		instrument = Device.new(channel_data.instrument, options, meter_id_instrument)
	end

	local channel = Channel.new(ch_index, channel_data, instrument, meter_id_channel)
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
		if v.roll then
			v.roll.ch_index = i
		end
	end
end

function build.new_device_data(device_key, options)
	local state = {}

	local index = 1

	for _, v in ipairs(options.parameters) do
		local w_name = v[1]
		local w_type = v[2] or w_name
		local w_options = v[3] or {}

		if w_type ~= "label" and w_type ~= "separator" then
			if w_type == "slider" then
				local sv = SliderValue.new(w_options)
				state[index] = sv.default
			elseif w_type == "selector" then
				state[index] = w_options.default or 1
			elseif w_type == "dropdown" then
				state[index] = w_options.default or 1
			elseif w_type == "toggle" then
				state[index] = w_options.default or false
			else
				error(w_type .. " not supported!")
			end

			assert(state[index] ~= nil)
			index = index + 1
		end
	end

	return { name = device_key, display_name = options.name, state = state, mute = false }
end

function build.new_channel_data(options)
	assert(options.name)

	local new = {
		effects = {},
		mute = false,
		armed = false,
		visible = true,
		lock = false,
		gain = 1.0,
		hue = find_hue(),
		name = options.name,
		master = options.master,
	}

	if options.instrument_key then
		new.instrument = build.new_device_data(options.instrument_key, options)
		new.notes = {}
		new.control = {}
	end

	return new
end

local function min_hue_dist(hue)
	-- calculate distance to closest hue that already exists
	local min_dist = 180.0
	for _, v in ipairs(project.channels) do
		-- distance in degrees
		local a = math.abs(hue - v.hue - 360.0 * math.floor(0.5 + (hue - v.hue) / 360.0))
		if a < min_dist then
			min_dist = a
		end
	end
	return min_dist
end

function find_hue()
	-- try some random hues, pick  the one that is furthest away from existing ones
	local hue = math.random() * 360.0
	local min_dist = min_hue_dist(hue)
	for _ = 1, 10 do
		local p_hue = math.random() * 360.0
		local p_min_dist = min_hue_dist(p_hue)
		if p_min_dist > min_dist then
			hue = p_hue
			min_dist = p_min_dist
		end
	end
	return hue
end

return build
