local audiolib = require("audiolib")
local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local midi = {}

local devices = {}

function midi.load(names)
	local device_handle = rtmidi.createIn()
	print("available midi input ports:")
	rtmidi.printPorts(device_handle)
	for _, v in ipairs(names) do
		midi.openDevice(v)
	end
end

function midi.openDevice(name)
	local device_handle = rtmidi.createIn()
	local port_n

	if name == "default" then
		port_n = 0
	else
		port_n = rtmidi.findPort(device_handle, name)
	end

	if port_n then
		rtmidi.openPort(device_handle, port_n)

		if device_handle.ok then
			rtmidi.ignoreTypes(device_handle, true, true, true)
			table.insert(devices, midi.newDevice(device_handle, "keyboard", port_n))
			return
		end
	end

	print("Couldn't open port: " .. name)
end

function midi.newDevice(handle, devicetype, n)
	local new = {}
	new.handle = handle
	new.devicetype = devicetype
	new.port = n
	new.voices = {}
	new.pitchbend = 2

	---temp
	new.offset = 0
	new.vel = 0

	if devicetype == "mpe" then
		new.pitchbend = 48
		-- for i = 1, 16 do
		-- 	new.voices[i] = { vel = 0.0, note = 49, offset = 0.0, y = 0, noteOn = false, noteOff = false }
		-- end
	end
	return new
end

function midi.update()
	for _, v in ipairs(devices) do
		midi.updateDevice(v)
	end
end

function midi.updateDevice(device)
	while true do
		local msg, s = rtmidi.getMessage(device.handle)
		if s == 0 then
			break
		end
		local event = midi.parse(msg, s)

		midi.handle_event(device, event)
	end
end

function midi.parse(msg, s)
	local status = bit.rshift(msg.data[0], 4)
	local channel = bit.band(msg.data[0], 15)

	local b = msg.data[1]
	local c = 0

	if s > 2 then
		c = msg.data[2]
	end

	local event = {}

	event.channel = channel

	if status == 9 and c > 0 then
		-- note on
		event.name = "note on"
		event.note = b
		event.vel = c / 127
	elseif status == 8 or (status == 9 and c == 0) then
		-- note off
		event.name = "note off"
		event.note = b
	elseif status == 13 then
		-- pressure
		event.name = "pressure"
		event.vel = b / 127
	elseif status == 14 then
		-- pitchbend
		event.name = "pitchbend"
		event.offset = (b + c * 128 - 8192) / 8192 -- [-1, 1]
	elseif status == 11 then
		event.name = "CC"
		event.cc = b
		event.y = c / 127
	end

	return event
end

function midi.handle_event_test(device, event)
	print(event.name)
end

function midi.handle_event(device, event)
	if event.name == "note on" then
		audiolib.send_noteOn(0, event.note + device.offset, event.vel)
		device.note = event.note
	elseif event.name == "note off" then
		if device.note then
			audiolib.send_CV(0, device.note + device.offset, 0)
		end
		device.note = nil
	elseif event.name == "pressure" then
		device.vel = event.vel
		if device.note then
			audiolib.send_CV(0, device.note + device.offset, device.vel)
		end
	elseif event.name == "pitchbend" then
		device.offset = 48 * event.offset
		if device.note then
			audiolib.send_CV(0, device.note + device.offset, device.vel)
		end
	else
		-- print(event.name)
	end
end

function midi.quit()
	for _, v in ipairs(devices) do
		rtmidi.closePort(v.handle)
	end
end

return midi
