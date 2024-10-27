local backend = require("backend")
local deviceList = require("device_list")
local Device = require("device")
local widgets = require("ui/widgets")
local SliderValue = require("ui/slider_value")
local log = require("log")

-- TODO: this should just be a channel struct and then some helper functions

local channelHandler = {}
channelHandler.list = {}

local function toNumber(x)
	if type(x) == "number" then
		return x
	elseif type(x) == "boolean" then
		return x and 1 or 0
	else
		error("unsupported type: " .. type(x))
	end
end

function channelHandler:sendParameters()
	for k, ch in ipairs(project_ui.channels) do
		for l, par in ipairs(ch.instrument.parameters) do
			local new_value = ch.instrument.state[l]
			local old_value = ch.instrument.state_old[l]
			if old_value ~= new_value then
				local value = toNumber(new_value)
				backend:sendParameter(k, 0, l, value)
				ch.instrument.state_old[l] = new_value
			end
		end

		for e, fx in ipairs(ch.effects) do
			for l, par in ipairs(fx.parameters) do
				local new_value = fx.state[l]
				local old_value = fx.state_old[l]
				if old_value ~= new_value then
					local value = toNumber(new_value)
					backend:sendParameter(k, e, l, value)
					fx.state_old[l] = new_value
				end
			end
		end
	end
end

local function init_device(name, options)
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

function channelHandler:add(name)
	local options = deviceList.instruments[name]
	if options then
		-- build state
		local new = {
			instrument = init_device(name, options),
			effects = {},
			mute = false,
			solo = false,
			armed = false,
			visible = true,
			lock = false,
			name = name .. " " .. #project.channels,
		}
		table.insert(project.channels, new)

		-- build UI
		local new_ui = { effects = {} }
		new_ui.instrument = Device:new(name, new.instrument.state, options)
		new_ui.widget = widgets.Channel:new()
		table.insert(project_ui.channels, new_ui)

		assert(#project_ui.channels == #project.channels)

		backend:addChannel(name)

		local ch_index = #project.channels
		selection.channel_index = ch_index

		-- new.midi_handler = MidiHandler:new(options.n_voices, new)
		self:addEffect(ch_index, "pan")

		return new
	else
		log.warn("Instrument not found: " .. name)
	end
end

function channelHandler:remove(ch_index)
	-- TODO: command
	table.remove(project.channels, ch_index)
	table.remove(project_ui.channels, ch_index)
	backend:removeChannel(ch_index)
end

function channelHandler:addEffect(ch_index, name)
	-- TODO: command

	-- TODO: insert at arbitrariy index / after selected device
	local index = 1
	local options = deviceList.effects[name]
	if options then
		local effect = init_device(name, options)
		table.insert(project.channels[ch_index].effects, index, effect)

		local effect_ui = Device:new(name, effect.state, options)
		table.insert(project_ui.channels[ch_index].effects, index, effect_ui)

		backend:addEffect(ch_index, name)

		selection.device_index = index
	else
		log.warn("Effect not found: " .. name)
	end
end

function channelHandler:removeEffect(ch_index, effect_index)
	-- TODO: command
	table.remove(project.channels[ch_index].effects, effect_index)
	table.remove(project_ui.channels[ch_index].effects, effect_index)
	backend:removeEffect(ch_index, effect_index)
end

function channelHandler:bypassEffect(ch, effect, bypass)
	-- TODO
	-- local ch_index = self:getChannelIndex(ch)
	-- backend:bypass(ch_index, effect_index, bypass)
end

function channelHandler:reorderEffect(ch_index, device_index, offset)
	if project.channels[ch_index] then
		local new_index = device_index + offset
		local n = #project.channels[ch_index].effects

		if device_index >= 1 and device_index <= n and new_index >= 1 and new_index <= n then
			local ch = project.channels[ch_index]
			local temp = table.remove(ch.effects, device_index)
			table.insert(ch.effects, new_index, temp)

			ch = project_ui.channels[ch_index]
			temp = table.remove(ch.effects, device_index)
			table.insert(ch.effects, new_index, temp)

			backend:reorderEffect(ch_index, device_index, new_index)

			if selection.channel_index == ch_index and selection.device_index == device_index then
				selection.device_index = new_index
			end
		end
	end
end

function channelHandler:mute(ch_index, mute)
	local ch = project.channels[ch_index]
	if mute then
		ch.solo = false
	end
	if ch.mute ~= mute then
		ch.mute = mute
		backend:sendMute(ch_index, mute)
	end
end

function channelHandler:solo(ch_index)
	local ch = project.channels[ch_index]
	if ch.solo then
		for i, v in ipairs(project.channels) do
			v.solo = false
			self:mute(i, false)
		end
	else
		for i, v in ipairs(project.channels) do
			if i == ch_index then
				v.solo = true
				self:mute(i, false)
			else
				v.solo = false
				self:mute(i, true)
			end
		end
	end
end

function channelHandler:armed(ch_index)
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
