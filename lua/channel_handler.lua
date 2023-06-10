local backend = require("backend")
local ParameterGroup = require("parameter_group")
local deviceList = require("device_list")

local channelHandler = {}
channelHandler.list = {}

function channelHandler:load()
	self.list = {}
end

function channelHandler:update()
	for k, ch in ipairs(self.list) do
		for l, par in ipairs(ch.instrument.parameters) do
			if par.dirty then
				backend:send_param(k - 1, 0, l - 1, par.v)
				par.dirty = false
			end
		end

		for e, fx in ipairs(ch.effects) do
			for l, par in ipairs(fx.parameters) do
				if par.dirty then
					backend:send_param(k - 1, e, l - 1, par.v)
					par.dirty = false
				end
			end
		end
	end
end

function channelHandler:add(name)
	if deviceList.instruments[name] then
		local new = {
			instrument = util.deepcopy(deviceList.instruments[name]),
			effects = {},
			visible = true,
			mute = false,
			solo = false,
			lock = false,
			armed = false,
		}

		new.instrument.name = name
		ParameterGroup.makeParameterGroups(new)

		table.insert(self.list, new)
		new.index = #self.list - 1
		new.name = name .. " " .. new.index

		backend:add_channel(new.instrument.index)
		selection.channel = new

		channelHandler:add_effect(new, "pan")

		return new
	else
		print("Instrument not found: " .. name)
	end
end

function channelHandler:add_effect(ch, name)
	if deviceList.effects[name] then
		local effect = util.deepcopy(deviceList.effects[name])

		table.insert(ch.effects, effect)

		effect.name = name

		ParameterGroup.addParameters(ch, effect)

		backend:add_effect(ch.index, deviceList.effects[name].index)

		return effect
	else
		print("Effect not found: " .. name)
	end
end

function channelHandler:mute(ch, mute)
	if ch.mute ~= mute then
		ch.mute = mute
		backend:send_mute(ch.index, mute)
	end
end

return channelHandler
