local backend = require("backend")
local deviceList = require("device_list")
local Device = require("device")
local MidiHandler = require("midi_handler")
local widgets = require("ui/widgets")
local SliderValue = require("ui/slider_value")

local channelHandler = {}

local function newDeviceData(name, options)
	local state = {}

	for i, v in ipairs(options.parameters) do
		local widget_type = v[2]
		local widget_options = v[3] or {}

		if widget_type == "slider" then
			local sv = SliderValue:new(widget_options)
			state[i] = sv.default
		elseif widget_type == "selector" then
			state[i] = widget_options.default or 1
		elseif widget_type == "toggle" then
			state[i] = widget_options.default or false
		else
			error(widget_type .. " not supported!")
		end

		assert(state[i] ~= nil)
	end

	return { name = name, state = state }
end

function channelHandler.buildChannel(channel)
	local options = deviceList.instruments[channel.instrument.name]
	assert(options)

	local channel_ui = { effects = {} }
	table.insert(ui_channels, channel_ui)
	channel_ui.instrument = Device:new(channel.name, channel.instrument.state, options)
	channel_ui.widget = widgets.Channel:new()

	backend:addChannel(channel.instrument.name)

	local ch_index = #ui_channels

	for _, v in ipairs(channel.effects) do
		channelHandler.buildEffect(ch_index, v)
	end

	channel_ui.midi_handler = MidiHandler:new(options.n_voices, ch_index)
	-- channelHandler.newEffect(ch_index, "pan")
end

function channelHandler.newChannel(name)
	local options = deviceList.instruments[name]
	assert(options)
	-- build state
	local channel = {
		instrument = newDeviceData(name, options),
		effects = {},
		mute = false,
		solo = false,
		armed = false,
		visible = true,
		lock = false,
		name = name .. " " .. #project.channels,
	}
	table.insert(project.channels, channel)

	channelHandler.buildChannel(channel)

	-- select it
	local ch_index = #project.channels
	selection.channel_index = ch_index

	return channel
end

function channelHandler.removeChannel(ch_index)
	-- TODO: command
	table.remove(project.channels, ch_index)
	table.remove(ui_channels, ch_index)
	backend:removeChannel(ch_index)
end

function channelHandler.buildEffect(ch_index, effect)
	local options = deviceList.effects[effect.name]
	assert(options)

	local effect_ui = Device:new(effect.name, effect.state, options)
	table.insert(ui_channels[ch_index].effects, effect_ui)

	backend:addEffect(ch_index, effect.name)
end

function channelHandler.newEffect(ch_index, name)
	-- TODO: command
	-- TODO: insert at arbitrariy index / after selected device

	local options = deviceList.effects[name]
	assert(options)

	local effect = newDeviceData(name, options)
	table.insert(project.channels[ch_index].effects, effect)

	channelHandler.buildEffect(ch_index, effect)

	-- select it
	selection.device_index = #ui_channels[ch_index].effects
end

function channelHandler.removeEffect(ch_index, effect_index)
	-- TODO: command
	table.remove(project.channels[ch_index].effects, effect_index)
	table.remove(ui_channels[ch_index].effects, effect_index)
	backend:removeEffect(ch_index, effect_index)
end

function channelHandler.bypassEffect(ch_index, effect_index, bypass)
	error("TODO")
	-- backend:bypass(ch_index, effect_index, bypass)
end

function channelHandler.reorderEffect(ch_index, device_index, offset)
	if project.channels[ch_index] then
		local new_index = device_index + offset
		local n = #project.channels[ch_index].effects

		if device_index >= 1 and device_index <= n and new_index >= 1 and new_index <= n then
			local ch = project.channels[ch_index]
			local temp = table.remove(ch.effects, device_index)
			table.insert(ch.effects, new_index, temp)

			ch = ui_channels[ch_index]
			temp = table.remove(ch.effects, device_index)
			table.insert(ch.effects, new_index, temp)

			backend:reorderEffect(ch_index, device_index, new_index)

			if selection.channel_index == ch_index and selection.device_index == device_index then
				selection.device_index = new_index
			end
		end
	end
end

function channelHandler.mute(ch_index, mute)
	local ch = project.channels[ch_index]
	if mute then
		ch.solo = false
	end
	if ch.mute ~= mute then
		ch.mute = mute
		backend:sendMute(ch_index, mute)
	end
end

function channelHandler.solo(ch_index)
	local ch = project.channels[ch_index]
	if ch.solo then
		for i, v in ipairs(project.channels) do
			v.solo = false
			channelHandler.mute(i, false)
		end
	else
		for i, v in ipairs(project.channels) do
			if i == ch_index then
				v.solo = true
				channelHandler.mute(i, false)
			else
				v.solo = false
				channelHandler.mute(i, true)
			end
		end
	end
end

function channelHandler.armed(ch_index)
	local ch = project.channels[ch_index]
	if ch.armed then
		ch.armed = false
	else
		for _, v in ipairs(project.channels) do
			v.armed = false
		end
		ch.armed = true
	end
end

return channelHandler
