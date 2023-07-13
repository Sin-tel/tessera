-- TODO: make this a nicer interface
-- make some kind of "device" object that you can poll

-- TODO: handle MPE

-- TODO: not stealing on retrigger same note

local Pitch = require("pitch")
local backend = require("backend")
local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

-- 0.01 = 40dB dynamic range
local VEL_MIN = 0.02
local LOG_RANGE = -math.log(VEL_MIN)

-- [0, 1] -> [0, 1]
local function velocity_curve(x)
	local v = x ^ 0.8
	local out = VEL_MIN * math.exp(LOG_RANGE * v)
	return out
end

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
	new.pres = 0
	new.age = 0
	new.note_on = false
	return new
end

function M.newDevice(handle, mpe, n)
	local new = {}
	new.handle = handle
	new.mpe = mpe
	new.port = n
	new.pitchbend_range = 2
	new.n_voices = 16
	new.offset = 0
	new.vel = 0
	new.queue = {}

	new.voices = {}
	for i = 1, new.n_voices do
		new.voices[i] = newVoice()
	end
	if mpe then
		new.pitchbend_range = 48
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
		event.pres = b / 127
	elseif status == 14 then
		event.name = "pitchbend"
		event.offset = (b + c * 128 - 8192) / 8192 -- [-1, 1]
	elseif status == 11 then
		event.name = "cc"
		event.cc = b
		event.y = c / 127
	end

	return event
end

function M.handleEventTest(device, event)
	print(event.name)
end

-- TODO: add a handler per instrument
function M.handleEvent(device, event)
	local channel_index = nil
	for i, ch in ipairs(channelHandler.list) do
		if ch.armed then
			channel_index = i
			break
		end
	end
	if not channel_index then
		return
	end

	if event.name == "note on" then
		-- voice stealing logic.
		-- if theres are voices free, use the oldest one,
		-- if not, steal a playing one.
		-- traditionally, priority is given to the oldest voice,
		-- instead, we steal the one closest in pitch, which allows for nicer voice leading.
		local playing_dist, playing_i = 10000, nil
		local released_age, released_i = -1, nil
		for i, v in ipairs(device.voices) do
			v.age = v.age + 1
			local dist = math.abs(v.note - event.note)
			if v.note_on then
				-- track note is on
				if dist < playing_dist then
					playing_i = i
					playing_dist = dist
				end
			else
				-- track is free
				if v.age > released_age then
					released_i = i
					released_age = v.age
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
		if voice.note_on then
			table.insert(device.queue, voice.note)
		end
		voice.note = event.note
		voice.vel = event.vel
		voice.pres = 0
		voice.note_on = true
		voice.age = 0
		local p = Pitch:fromMidi(voice.note)
		backend:sendNote(channel_index, p.pitch + voice.offset, velocity_curve(voice.vel), new_i)
	elseif event.name == "note off" then
		local get_i
		for i, v in ipairs(device.voices) do
			if v.note == event.note and v.note_on then
				get_i = i
				break
			end
		end
		if get_i == nil then
			-- voice was already dead
			for i, v in ipairs(device.queue) do
				if v == event.note then
					table.remove(device.queue, i)
					break
				end
			end
			return
		end
		local voice = device.voices[get_i]

		if #device.queue == 0 then
			voice.note_on = false
			if not device.sustain then
				-- note off pitches are ignored but we send the correct one anyway
				local p = Pitch:fromMidi(voice.note)
				backend:sendNote(channel_index, p.pitch + voice.offset, 0, get_i)
			end
		else
			-- pop last note in queue
			voice.note = table.remove(device.queue)
			local p = Pitch:fromMidi(voice.note)
			backend:sendNote(channel_index, p.pitch + voice.offset, velocity_curve(voice.vel), get_i)
		end
	elseif event.name == "pitchbend" then
		for i, v in ipairs(device.voices) do
			v.offset = device.pitchbend_range * event.offset
			if v.note_on then
				backend:sendCv(channel_index, v.note + v.offset, v.pres, i)
			end
		end
	elseif event.name == "pressure" then
		for i, v in ipairs(device.voices) do
			v.pres = event.pres
			if v.note_on then
				backend:sendCv(channel_index, v.note + v.offset, v.pres, i)
			end
		end
	elseif event.name == "cc" then
		if event.cc == 64 then
			-- sustain pedal
			if event.y > 0 then
				device.sustain = true
			else
				device.sustain = false
				device.queue = {}
				for i, v in ipairs(device.voices) do
					if not v.note_on then
						v.note_on = false
						-- note off pitches are ignored but we send the correct one anyway
						local p = Pitch:fromMidi(v.note)
						backend:sendNote(channel_index, p.pitch + v.offset, 0, i)
					end
				end
			end
		end
	end
end

function M.quit()
	for _, v in ipairs(devices) do
		rtmidi.closePort(v.handle)
	end
end

return M
