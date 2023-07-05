-- TODO: make this a nicer interface
-- make some kind of "device" object that you can poll
-- TODO: handle mono / poly playing

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

local function newVoice()
	local new = {}
	new.channel = 0
	new.note = 0
	new.offset = 0
	new.vel = 0
	new.age = 0
	new.note_on = false
	return new
end

function M.newDevice(handle, mpe, n)
	local new = {}
	new.handle = handle
	new.mpe = mpe
	new.port = n
	new.pitchbend = 2

	new.n_voices = 4

	---temp
	new.offset = 0
	new.vel = 0

	new.voices = {}
	for i = 1, new.n_voices do
		new.voices[i] = newVoice()
	end
	if mpe then
		new.pitchbend = 48
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

		M.handleEventPoly(device, event)
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
		event.pres = b / 127
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

-- TODO: remove
local index = 0

function M.handleEventPoly(device, event)
	local channel_index = nil
	for i, ch in ipairs(channelHandler.list) do
		if ch.armed then
			channel_index = i - 1
			break
		end
	end
	if not channel_index then
		return
	end

	if event.name == "note on" then
		local playing_age, playing_i = 10000, nil
		local released_age, released_i = -1, nil
		for j, b in ipairs(device.voices) do
			b.age = b.age + 1
			if b.note_on then
				-- track note is on
				-- TODO: find closest instead of newest
				if b.age < playing_age then
					playing_i = j
					playing_age = b.age
				end
			else
				-- track is free
				if b.age > released_age then
					released_i = j
					released_age = b.age
				end
			end
		end
		local new_i
		if released_i ~= nil then
			new_i = released_i
		else
			new_i = playing_i
		end

		assert(new_i ~= nil)

		local voice = device.voices[new_i]
		voice.note = event.note
		voice.vel = event.vel
		voice.pres = 0
		voice.note_on = true
		voice.age = 0
		backend:sendNote(channel_index, voice.note + voice.offset, voice.vel, new_i - 1)
	elseif event.name == "note off" then
		local get_i
		for i, v in ipairs(device.voices) do
			if v.note == event.note and v.note_on then
				get_i = i
				break
			end
		end
		if get_i == nil then
			--voice was already dead
			return
		end
		local voice = device.voices[get_i]

		voice.note_on = false

		backend:sendNote(channel_index, voice.note + voice.offset, 0, get_i - 1)

		-- if device.note and #device.voices == 1 then
		-- 	backend:sendNote(channel_index, device.note + device.offset, 0)
		-- 	device.note = nil
		-- elseif get_i == #device.voices then
		-- 	device.note = device.voices[get_i - 1]
		-- 	backend:sendNote(channel_index, device.note + device.offset, device.vel)
		-- end
		-- table.remove(device.voices, get_i)

		-- elseif event.name == "pressure" then
		-- 	device.pres = event.pres
		-- 	if device.note then
		-- 		backend:sendCv(channel_index, device.note + device.offset, device.pres)
		-- 	end
		-- elseif event.name == "pitchbend" then
		-- 	device.offset = device.pitchbend * event.offset
		-- 	if device.note then
		-- 		backend:sendCv(channel_index, device.note + device.offset, device.pres)
		-- 	end
		-- else
		-- print(event.name)
	end

	-- return signal
end

function M.handleEvent(device, event)
	local channel_index = nil
	for i, ch in ipairs(channelHandler.list) do
		if ch.armed then
			channel_index = i - 1
			break
		end
	end
	if not channel_index then
		return
	end
	if event.name == "note on" then
		table.insert(device.voices, event.note)
		device.note = event.note
		device.vel = event.vel
		device.pres = 0
		backend:sendNote(channel_index, device.note + device.offset, device.vel, 0)
	elseif event.name == "note off" then
		local get_i
		for i, v in ipairs(device.voices) do
			if v == event.note then
				get_i = i
				break
			end
		end
		if device.note and #device.voices == 1 then
			backend:sendNote(channel_index, device.note + device.offset, 0)
			device.note = nil
		elseif get_i == #device.voices then
			device.note = device.voices[get_i - 1]
			backend:sendNote(channel_index, device.note + device.offset, device.vel)
		end
		table.remove(device.voices, get_i)
	elseif event.name == "pressure" then
		device.pres = event.pres
		if device.note then
			backend:sendCv(channel_index, device.note + device.offset, device.pres)
		end
	elseif event.name == "pitchbend" then
		device.offset = device.pitchbend * event.offset
		if device.note then
			backend:sendCv(channel_index, device.note + device.offset, device.pres)
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
