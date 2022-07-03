local audiolib = require("audiolib")

channels = {}
channels.list = {}

function channels.init()
	channels.list = {}
end

function channels.update()
	for k, ch in ipairs(channels.list) do
		if ch.channel[1].dirty or ch.channel[2].dirty then
			audiolib.send_pan(k-1, {ch.channel[1].v, ch.channel[2].v})
			ch.channel[1].dirty = false
			ch.channel[2].dirty = false
		end

		for l, par in ipairs(ch.instrument.parameters) do
			if par.dirty then
				audiolib.send_param(k-1, 0, l-1, par.v)
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

function channels.add(name)
	if devicelist.instruments[name] then
		local new = {
			channel = deepcopy(devicelist.channel),
			instrument = deepcopy(devicelist.instruments[name]),
			effects = {},
		}

		new.instrument.name = name
		new.parametergroups = ParameterView:makeparametergroups(new)

		table.insert(channels.list, new)
		new.index = #channels.list - 1
		new.name = name .. " " .. new.index

		new.visible = true
		new.mute = false
		new.solo = false
		new.lock = false
		new.armed = false

		audiolib.add_channel(new.instrument.index)
		selection.channel = new
	else
		print("instrument not found: " .. name)
	end
end

function channels.mute(ch, mute)
	if ch.mute ~= mute then
		ch.mute = mute
		audiolib.send_mute(ch.index, mute)
	end
end