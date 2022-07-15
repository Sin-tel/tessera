local audiolib = require("audiolib")

local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local midi = {}

local devices = {}

local function newdevice(handle, devicetype, n)
	local new = {}
	new.handle = handle
	new.devicetype = devicetype
	new.port = n
	new.voices = {}
	new.pitchbend = 2

	if devicetype == "mpe" then
		new.pitchbend = 48
		-- for i = 1, 16 do
		-- 	new.voices[i] = { vel = 0.0, note = 49, offset = 0.0, y = 0, noteOn = false, noteOff = false }
		-- end
	end
	return new
end

function midi.load(names)
	local device_handle = rtmidi.createIn()
	print("available midi input ports:")
	rtmidi.printPorts(device_handle)
	for _, v in ipairs(names) do
		midi.openDevice(v)
	end
end

function midi.update()
	for _, v in ipairs(devices) do
		midi.updateDevice(v)
	end
end

function midi.openDevice(name)
	local device_handle = rtmidi.createIn()
	local port_n

	if name ~= "default" then
		port_n = rtmidi.findPort(device_handle, name)
	end

	if not port_n then
		print("Opening default midi port (0)")
		port_n = 0
	end

	rtmidi.openPort(device_handle, port_n)

	if device_handle.ok then
		rtmidi.ignoreTypes(device_handle, true, true, true)
		table.insert(devices, newdevice(device_handle, "keyboard", port_n))
	else
		print("Couldn't open port: " .. name)
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

local function handle_midi_test(device, event)
	print(event.name)
end

local function handle_midi_mono(device, event)
	print(event.name)
end

function midi.updateDevice(device)
	while true do
		local msg, s = rtmidi.getMessage(device.handle)
		if s == 0 then
			break
		end
		local event = midi.parse(msg, s)

		handle_midi_mono(device, event)
	end
end

function midi.quit()
	for _, v in ipairs(devices) do
		rtmidi.closePort(v.handle)
	end
end

return midi
