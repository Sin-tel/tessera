local audiolib = require("audiolib")

channelHandler = {}
channelHandler.list = {}

function channelHandler:load()
	self.list = {}
end

function channelHandler:update()
	for k, ch in ipairs(self.list) do
		if ch.parameters[1].dirty or ch.parameters[2].dirty then
			audiolib.send_pan(k - 1, { ch.parameters[1].v, ch.parameters[2].v })
			ch.parameters[1].dirty = false
			ch.parameters[2].dirty = false
		end

		for l, par in ipairs(ch.instrument.parameters) do
			if par.dirty then
				audiolib.send_param(k - 1, 0, l - 1, par.v)
				par.dirty = false
			end
		end

		-- for e, fx in ipairs(ch.effects) do
		-- 	for l, par in ipairs(e.parameters) do
		-- 		audiolib.send_param(k-1, e, l-1, par.v)
		-- 	end
		-- end
	end
end

function channelHandler:add(name)
	if deviceList.instruments[name] then
		local new = {
			parameters = deepcopy(deviceList.channel),
			instrument = deepcopy(deviceList.instruments[name]),
			effects = {},
			visible = true,
			mute = false,
			solo = false,
			lock = false,
			armed = false,
		}

		new.instrument.name = name
		new.parameterGroups = parameterView:makeParameterParameterGroups(new)

		table.insert(self.list, new)
		new.index = #self.list - 1
		new.name = name .. " " .. new.index

		audiolib.add_channel(new.instrument.index)
		selection.channel = new

		return new
	else
		print("instrument not found: " .. name)
	end
end

function channelHandler:mute(ch, mute)
	if ch.mute ~= mute then
		ch.mute = mute
		audiolib.send_mute(ch.index, mute)
	end
end
