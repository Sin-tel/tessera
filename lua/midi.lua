local backend = require("backend")
local log = require("log")

local midi = {}
local devices = {}

function midi.load(input_ports)
	backend:midiListPorts()

	for _, v in ipairs(input_ports) do
		-- TODO: 'default' should just open first port

		local name, index = backend:midiOpenConnection(v.name)
		if name then
			assert(not devices[index])
			devices[index] = midi.newDevice(v, name, index)
		end
	end
end

function midi.newDevice(settings, name, index)
	local new = {}
	new.index = index
	new.mpe = settings.mpe
	new.name = name
	new.pitchbend_range = 2
	if new.mpe then
		new.pitchbend_range = 48
	end
	return new
end

function midi.update()
	for _, device in pairs(devices) do
		midi.updateDevice(device)
	end
end

function midi.updateDevice(device)
	-- TODO: remove
	local handler
	for i, ch in ipairs(channelHandler.list) do
		if ch.armed then
			handler = ch.midi_handler
			break
		end
	end

	local events = backend:midiPoll(device.index)

	if handler then
		for _, event in ipairs(events) do
			handler:event(device, event)
		end
	end
end

return midi
