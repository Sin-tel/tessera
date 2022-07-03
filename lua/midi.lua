local audiolib = require("audiolib")

local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local midi = {}

local function newdevice(handle, devicetype, n)
	local voices = {}
	if devicetype == "mpe" then
		for i = 1,16 do
			voices[i] = {vel = 0.0, note = 49, offset = 0.0, y = 0, noteOn = false, noteOff = false}
		end
	end
	return {
		handle = handle,
		devicetype = devicetype,
		port = n,
		voices = voices,
	}
end

function midi.load(name)
	local device_handle = rtmidi.createIn()
	rtmidi.printPorts(device_handle)


	local port_n = false
	if name ~= "default" then
		port_n = rtmidi.findPort(device_handle, name)
	end

	if port_n == false then
		print("Opening default midi port (0)")
		port_n = 0
	end

	rtmidi.openPort(device_handle, port_n)


	rtmidi.ignoreTypes(device_handle, true, true, true)

	return newdevice(device_handle, "keyboard", port_n)
end

function midi.update(device, func)
	while true do
		local msg, s = rtmidi.getMessage(device.handle)
		if s == 0 then
			break
		end

		local status = bit.rshift(msg.data[0], 4)
		local channel = bit.band(msg.data[0], 15)	

		local index = channel + 1

		local b = msg.data[1]
		local c = 0
		if s > 2 then
			c = msg.data[2]
		end

		-- print(status, b, c)
		local event = {}

		event.channel = channel

		if status == 9 and c > 0 then 
			-- note on
			event.name = "note on"
			event.note = b
			event.vel = c/127
		elseif status == 8 or (status == 9 and c == 0) then 
			-- note off
			event.name = "note off"
			event.note = b
		elseif status == 13 then 
			-- pressure
			event.name = "pressure"
			event.vel = b/127
		elseif status == 14 then 
			-- pitchbend
			event.name = "pitchbend"
			if device.devicetype == "mpe" then
				event.offset = 48*(b+c*128 - 8192)/8192 -- MPE
			else
				event.offset = 2*(b+c*128 - 8192)/8192
			end
		elseif status == 11 then  
			-- CC y
			event.name = "CC"
			event.cc = b
			event.y = c/127
			-- if b == 74 then
			--
			-- end
		end

		func(event)
	end
end

function midi.close(device)
	rtmidi.closePort(device.handle)
end

function midi.draw(device) 
	for i,v in ipairs(device.voices) do
		love.graphics.ellipse("fill", (v.note+v.offset)*10, 500-200*v.y, v.vel*20)
	end
end


function handle_midi_test(event)
	print(event.name)
end


polyphony = 1
n_index = 1
tracks = {}

function newTrack()
	local new = {}
	new.channel = 0
	new.note = 0
	new.offset = 0
	new.vel = 0
	new.age = 0
	new.isPlaying = false
	return new
end

for i = 1, polyphony do
	tracks[i] = newTrack()
end

-- polyphonic note stealing logic
-- doesnt work well for mono
function handle_midi(event)
	local index = -1

	for i, ch in ipairs(channels.list) do
		if ch.armed then
			index = ch.index
		end
	end

	if index ~= -1 then

		if event.name == "note on" then	
			print(event.note)
			local oldestPlaying_age, oldestPlaying_i = -1, -1
			local oldestReleased_age, oldestReleased_i = -1, -1
			for j,b in ipairs(tracks) do
				b.age = b.age+1
				if b.isPlaying then
					-- track note is on
					if b.age>oldestPlaying_age then
						oldestPlaying_i = j
						oldestPlaying_age = b.age
					end
				else
					-- track is free
					if b.age>oldestReleased_age then
						oldestReleased_i = j
						oldestReleased_age = b.age
					end
				end
			end
			local new_i = nil
			if oldestReleased_i ~= -1 then
				new_i = oldestReleased_i
			else
				new_i = oldestPlaying_i
			end
			tracks[new_i].channel = event.channel
			tracks[new_i].note = event.note
			tracks[new_i].age = 0
			tracks[new_i].isPlaying = true
			tracks[new_i].vel = event.vel

			audiolib.send_noteOn(index, {tracks[new_i].note + tracks[new_i].offset, event.vel})
		end

		if event.name == "note off" then
			for j,b in ipairs(tracks) do
				if b.note == event.note then
					print("noteoff")
					b.isPlaying = false
					b.age = 0
					b.vel = 0
					audiolib.send_CV(index, {b.note + b.offset, 0})
					-- b.note = -1

					break
				end
			end
		end

		if event.name == "CC" then
			for j,b in ipairs(tracks) do
				if b.channel == event.channel then
					if event.cc == 74 then
						b.vel = event.y
						audiolib.send_CV(index, {b.note + b.offset, b.vel})
						break
					end
				end
			end
		end

		if event.name == "pitchbend" then
			for j,b in ipairs(tracks) do
				if b.channel == event.channel then
					b.offset = event.offset
					audiolib.send_CV(index, {b.note + b.offset, b.vel})
					break
				end
			end
		end
	end
end

return midi