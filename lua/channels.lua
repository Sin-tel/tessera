local audiolib = require("audiolib")

channels = {}
channels.list = {}

function channels.init()
	channels.list = {}
end

function channels.update()
	--TODO: only send when changed? (dirty flag or sth)
	for k, ch in ipairs(channels.list) do
		audiolib.send_pan(k-1, {ch.channel[1].v, ch.channel[2].v})

		for l, par in ipairs(ch.instrument.parameters) do
			audiolib.send_param(k-1, 0, l-1, par.v)
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

		new.index = #channels.list
		table.insert(channels.list, new)


		audiolib.add_channel(new.instrument.index)

		selection.channel = new
	else
		print("instrument not found: " .. name)
	end
end