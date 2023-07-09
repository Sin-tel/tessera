local backend = require("backend")
local deviceList = require("device_list")
local Device = require("device")
local widgets = require("ui/widgets")

-- TODO: this should just be a channel struct and then some helper functions

local channelHandler = {}
channelHandler.list = {}

function channelHandler:load()
	self.list = {}
end

function channelHandler:sendParameters()
	for k, ch in ipairs(self.list) do
		for l, par in ipairs(ch.instrument.parameters) do
			local value = par.widget:getFloat()
			if value then
				backend:sendParameter(k - 1, 0, l - 1, value)
			end
		end

		for e, fx in ipairs(ch.effects) do
			for l, par in ipairs(fx.parameters) do
				local value = par.widget:getFloat()
				if value then
					backend:sendParameter(k - 1, e, l - 1, value)
				end
			end
		end
	end
end

function channelHandler:add(name)
	if deviceList.instruments[name] then
		local new = {
			instrument = Device:new(name, deviceList.instruments[name]),
			effects = {},
			mute = false,
			solo = false,
			armed = false,
			visible = true,
			lock = false,
		}

		table.insert(self.list, new)
		new.index = #self.list - 1 -- Rust backend index starts at zero
		new.name = name .. " " .. new.index

		new.widget = widgets.Channel:new(new)

		backend:addChannel(new.instrument.index)
		selection.channel = new

		self:addEffect(new, "pan")

		return new
	else
		print("Instrument not found: " .. name)
	end
end

function channelHandler:remove(index)
	table.remove(self.list, index)
	backend:removeChannel(index)
end

function channelHandler:addEffect(ch, name)
	if deviceList.effects[name] then
		local effect = Device:new(name, deviceList.effects[name])

		table.insert(ch.effects, 1, effect)

		backend:addEffect(ch.index, effect.index)

		return effect
	else
		print("Effect not found: " .. name)
	end
end

function channelHandler:removeEffect(ch, index)
	table.remove(ch.effects, index)
	backend:removeEffect(index)
end

function channelHandler:bypassEffect(channel_index, effect_index, bypass)
	-- TODO
	backend:bypassEffect(channel_index, effect_index, bypass)
end

function channelHandler:reorderEffect(channel_index, old_index, new_index)
	local temp = table.remove(ch.effects, old_index)
	table.insert(ch.effects, new_index)
	backend:reorderEffect(channel_index, old_index, new_index)
end

function channelHandler:mute(ch, mute)
	if mute then
		ch.solo = false
	end
	if ch.mute ~= mute then
		ch.mute = mute
		backend:sendMute(ch.index, mute)
	end
end

function channelHandler:solo(ch)
	if ch.solo then
		for _, ch in ipairs(self.list) do
			ch.solo = false
			self:mute(ch, false)
		end
	else
		for _, ch in ipairs(self.list) do
			ch.solo = false
			self:mute(ch, true)
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
