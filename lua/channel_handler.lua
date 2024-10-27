local backend = require("backend")
local deviceList = require("device_list")
local Device = require("device")
local widgets = require("ui/widgets")
local MidiHandler = require("midi_handler")
local log = require("log")

-- TODO: this should just be a channel struct and then some helper functions

local channelHandler = {}
channelHandler.list = {}

function channelHandler:load()
	self.list = {}
end

function channelHandler:sendParameters()
	-- for k, ch in ipairs(self.list) do
	-- 	for l, par in ipairs(ch.instrument.parameters) do
	-- 		local value = par.widget:getFloat()
	-- 		if value then
	-- 			backend:sendParameter(k, 0, l, value)
	-- 		end
	-- 	end

	-- 	for e, fx in ipairs(ch.effects) do
	-- 		for l, par in ipairs(fx.parameters) do
	-- 			local value = par.widget:getFloat()
	-- 			if value then
	-- 				backend:sendParameter(k, e, l, value)
	-- 			end
	-- 		end
	-- 	end
	-- end
end

function channelHandler:add(name)
	local options = deviceList.instruments[name]
	if options then
		local new = {
			instrument = Device:new(name, options),
			effects = {},
			mute = false,
			solo = false,
			armed = false,
			visible = true,
			lock = false,
		}

		table.insert(self.list, new)
		new.name = name .. " " .. #self.list

		new.widget = widgets.Channel:new(new)

		backend:addChannel(new.instrument.number)
		selection.channel = new

		new.midi_handler = MidiHandler:new(options.n_voices, new)

		self:addEffect(new, "pan")

		return new
	else
		log.warn("Instrument not found: " .. name)
	end
end

function channelHandler:getChannelIndex(ch)
	for i, v in ipairs(self.list) do
		if v == ch then
			return i
		end
	end

	error("channel not found" .. ch.name)
end

function channelHandler:getEffectIndex(ch, effect)
	for i, v in ipairs(ch.effects) do
		if v == effect then
			return i
		end
	end
end

function channelHandler:remove(ch)
	local ch_index = self:getChannelIndex(ch)
	table.remove(self.list, ch_index)
	backend:removeChannel(ch_index)
end

function channelHandler:addEffect(ch, name)
	if deviceList.effects[name] then
		local ch_index = self:getChannelIndex(ch)

		local effect = Device:new(name, deviceList.effects[name])

		table.insert(ch.effects, 1, effect)
		backend:addEffect(ch_index, effect.number)

		return effect
	else
		log.warn("Effect not found: " .. name)
	end
end

function channelHandler:removeEffect(ch, device)
	local effect_index = self:getEffectIndex(ch, device)
	local ch_index = self:getChannelIndex(ch)
	table.remove(ch.effects, effect_index)
	backend:removeEffect(ch_index, effect_index)
end

function channelHandler:bypassEffect(ch, effect, bypass)
	-- TODO
	-- local ch_index = self:getChannelIndex(ch)
	-- backend:bypass(ch_index, effect_index, bypass)
end

function channelHandler:reorderEffect(ch, device, offset)
	local old_index = self:getEffectIndex(ch, device)
	if old_index then
		local new_index = old_index + offset

		local n = #ch.effects
		if old_index >= 1 and old_index <= n and new_index >= 1 and new_index <= n then
			local temp = table.remove(ch.effects, old_index)
			table.insert(ch.effects, new_index, temp)

			local ch_index = self:getChannelIndex(ch)
			backend:reorderEffect(ch_index, old_index, new_index)
		end
	end
end

function channelHandler:mute(ch, mute)
	if mute then
		ch.solo = false
	end
	if ch.mute ~= mute then
		ch.mute = mute
		local ch_index = self:getChannelIndex(ch)
		backend:sendMute(ch_index, mute)
	end
end

function channelHandler:solo(ch)
	if ch.solo then
		for _, v in ipairs(self.list) do
			v.solo = false
			self:mute(v, false)
		end
	else
		for _, v in ipairs(self.list) do
			v.solo = false
			self:mute(v, true)
		end
		ch.solo = true
		self:mute(ch, false)
	end
end

function channelHandler:armed(ch)
	if ch.armed then
		ch.armed = false
	else
		for _, v in ipairs(self.list) do
			v.armed = false
		end
		ch.armed = true
	end
end

return channelHandler
