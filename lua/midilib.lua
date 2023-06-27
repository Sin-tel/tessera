-- TODO: make this a nicer interface
-- make some kind of "device" object that you can poll

local backend = require("backend")
local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local M = {}

local devices = {}

function M.load(devicelist)
	local device_handle = rtmidi.createIn()
	print("available midi input ports:")
	rtmidi.printPorts(device_handle)

	for _, v in ipairs(devicelist) do
		M.openDevice(v)
	end
end

function M.openDevice(v)
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
			table.insert(devices, M.newDevice(device_handle, v.mpe, port_n))
			return
		end
	end

	print("Couldn't open port: " .. v.name)
end

function M.newDevice(handle, mpe, n)
	local new = {}
	new.handle = handle
	new.mpe = mpe
	new.port = n
	new.voices = {}
	new.pitchbend = 2

	---temp
	new.offset = 0
	new.vel = 0

	if mpe then
		new.pitchbend = 48
		-- for i = 1, 16 do
		-- 	new.voices[i] = { vel = 0.0, note = 49, offset = 0.0, y = 0, noteOn = false, noteOff = false }
		-- end
	end
	return new
end

function M.update()
	for _, v in ipairs(devices) do
		M.updateDevice(v)
	end
end

function M.updateDevice(device)
	while true do
		local msg, s = rtmidi.getMessage(device.handle)
		if s == 0 then
			break
		end
		local event = M.parse(msg, s)

		M.handleEvent(device, event)
	end
end

function M.parse(msg, s)
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
		event.name = "note on"
		event.note = b
		event.vel = c / 127
	elseif status == 8 or (status == 9 and c == 0) then
		event.name = "note off"
		event.note = b
	elseif status == 13 then
		event.name = "pressure"
		event.vel = b / 127
	elseif status == 14 then
		event.name = "pitchbend"
		event.offset = (b + c * 128 - 8192) / 8192 -- [-1, 1]
	elseif status == 11 then
		event.name = "CC"
		event.cc = b
		event.y = c / 127
	end

	return event
end

function M.handleEventTest(device, event)
	print(event.name)
end

function M.handleEvent(device, event)
	if event.name == "note on" then
		table.insert(device.voices, event.note)
		device.note = event.note
		device.vel = event.vel
		backend:sendNoteOn(0, device.note + device.offset, device.vel, 0)
	elseif event.name == "note off" then
		local get_i
		for i, v in ipairs(device.voices) do
			if v == event.note then
				get_i = i
				break
			end
		end
		if device.note and #device.voices == 1 then
			backend:sendCv(0, device.note + device.offset, 0)
			device.note = nil
		elseif get_i == #device.voices then
			device.note = device.voices[get_i - 1]
			backend:sendNoteOn(0, device.note + device.offset, device.vel)
		end
		table.remove(device.voices, get_i)
	elseif event.name == "pressure" then
		device.vel = event.vel
		if device.note then
			backend:sendCv(0, device.note + device.offset, device.vel)
		end
	elseif event.name == "pitchbend" then
		device.offset = device.pitchbend * event.offset
		if device.note then
			backend:sendCv(0, device.note + device.offset, device.vel)
		end
	else
		-- print(event.name)
	end

	-- return signal
end

function M.quit()
	for _, v in ipairs(devices) do
		rtmidi.closePort(v.handle)
	end
end

return M
