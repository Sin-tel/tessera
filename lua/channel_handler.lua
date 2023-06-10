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
		if ch.parameters[1].dirty or ch.parameters[2].dirty then
			backend:send_pan(k - 1, ch.parameters[1].v, ch.parameters[2].v)
			ch.parameters[1].dirty = false
			ch.parameters[2].dirty = false
		end

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
			parameters = util.deepcopy(deviceList.channel),
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

		backend:add_channel(new.instrument.number)
		selection.channel = new

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

		backend:add_effect(ch.index, deviceList.effects[name].number)

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
