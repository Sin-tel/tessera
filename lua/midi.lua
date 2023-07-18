-- TODO: make this a nicer interface
-- make some kind of "device" object that you can poll

local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local midi = {}

local devices = {}

function midi.load(devicelist)
	local device_handle = rtmidi.createIn()
	print("available midi input ports:")
	rtmidi.printPorts(device_handle)

	for _, v in ipairs(devicelist) do
		midi.openDevice(v)
	end
end

function midi.openDevice(v)
	local device_handle = rtmidi.createIn()
	local port_n

	if v.name == "default" then
		port_n = 0
	else
		port_n = rtmidi.findPort(device_handle, v.name)
	end

	if port_n then
		rtmidi.openPort(device_handle, port_n)

		if device_handle.ok then
			rtmidi.ignoreTypes(device_handle, true, true, true)
			table.insert(devices, midi.newDevice(device_handle, v.mpe, port_n))
			return
		end
	end

	print("Couldn't open port: " .. v.name)
end

function midi.newDevice(handle, mpe, n)
	local new = {}
	new.handle = handle
	new.mpe = mpe
	new.port = n
	new.pitchbend_range = 2

	if mpe then
		new.pitchbend_range = 48
	end
	return new
end

function midi.update()
	for _, v in ipairs(devices) do
		midi.updateDevice(v)
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

	while true do
		local message, size = rtmidi.getMessage(device.handle)
		if size == 0 then
			break
		end
		local event = midi.parse(message, size)

		if handler then
			handler:event(device, event)
		end
	end
end

function midi.parse(message, size)
	local status = bit.rshift(message.data[0], 4)
	local channel = bit.band(message.data[0], 15)

	local a = message.data[1]
	local b = 0

	if size > 2 then
		b = message.data[2]
	end

	local event = {}

	event.channel = channel

	if status == 9 and b > 0 then
		event.name = "note on"
		event.note = a
		event.vel = b / 127
	elseif status == 8 or (status == 9 and b == 0) then
		event.name = "note off"
		event.note = a
	elseif status == 13 then
		event.name = "pressure"
		event.pres = a / 127
	elseif status == 14 then
		event.name = "pitchbend"
		event.offset = (a + b * 128 - 8192) / 8192 -- [-1, 1]
	elseif status == 11 then
		event.name = "cc"
		event.cc = a
		event.y = b / 127
	end

	return event
end

function midi.quit()
	for _, v in ipairs(devices) do
		rtmidi.closePort(v.handle)
	end
end

return midi
