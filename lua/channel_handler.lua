local backend = require("backend")
local deviceList = require("device_list")
local Device = require("device")

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
			visible = true,
			mute = false,
			solo = false,
			lock = false,
			armed = false,
		}

		table.insert(self.list, new)
		new.index = #self.list - 1 -- Rust backend index starts at zero
		new.name = name .. " " .. new.index

		backend:addChannel(new.instrument.index)
		selection.channel = new

		channelHandler:addEffect(new, "pan")

		return new
	else
		print("Instrument not found: " .. name)
	end
end

function channelHandler:addEffect(ch, name)
	if deviceList.effects[name] then
		local effect = Device:new(name, deviceList.effects[name])

		table.insert(ch.effects, effect)

		backend:addEffect(ch.index, effect.index)

		return effect
	else
		print("Effect not found: " .. name)
	end
end

function channelHandler:mute(ch, mute)
	if ch.mute ~= mute then
		ch.mute = mute
		backend:sendMute(ch.index, mute)
	end
end

return channelHandler
