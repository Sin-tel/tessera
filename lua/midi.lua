local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local midi = {}

local function device(handle, mpe,n)
	local voices = {}
	if mpe then
		for i = 1,16 do
			voices[i] = {vel = 0.0, note = 49, offset = 0.0, y = 0, noteOn = false, noteOff = false}
		end
	end
	return {
		handle = handle,
		mpe = mpe,
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

	return device(device_handle, true, port_n)
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
			event.offset = 48*(b+c*128 - 8192)/8192
		elseif status == 11 then  
			-- CC y
			event.name = "CC"
			if b == 74 then
				event.y = c/127
			end
		end

		func(event)
	end
end

function midi.draw(device) 
	for i,v in ipairs(device.voices) do
		love.graphics.ellipse("fill", (v.note+v.offset)*10, 500-200*v.y, v.vel*20)
	end
end


polyphony = 4
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

function handle_midi(event)
	if event.name == "note on" then	
		local oldestPlaying_age, oldestPlaying_i = -1, -1
		local oldestReleased_age, oldestReleased_i = -1, -1
		for j,b in ipairs(tracks) do
			b.age = b.age+1
			print(b.isPlaying)
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
		-- print(oldestPlaying_age, oldestPlaying_i)
		-- print(oldestReleased_age, oldestReleased_i)
		local new_i = nil
		if oldestReleased_i ~= -1 then
			new_i = oldestReleased_i
		else
			new_i = oldestPlaying_i
		end
		tracks[new_i].channel = i
		tracks[new_i].note = v.note
		tracks[new_i].age = 0
		tracks[new_i].isPlaying = true
		audiolib.send_noteOn(new_i - 1, {v.note + v.offset, v.vel});
	end

	if event.name == "note off" then
		v.noteOff = false;
		
		for j,b in ipairs(tracks) do
			-- free note
			-- print(b.channel, i)
			print(b.note, v.note)
			if b.channel == i and b.note == v.note then
				print("noteoff")
				tracks[j].isPlaying = false
				tracks[j].age = 0
				tracks[j].note = -1

				audiolib.send_CV(j-1, {v.note + v.offset, 0});
				break
			end
		end
	end

	if event.name == "CC" then
		-- update cv
		for j,b in ipairs(tracks) do
			if b.channel == i and b.note == v.note then
				audiolib.send_CV(j-1, {v.note + v.offset, v.vel});
			end
		end
	end

	if event.name == "pitchbend" then
		-- update pitchbend
		for j,b in ipairs(tracks) do
			if b.channel == i and b.note == v.note then
				audiolib.send_CV(j-1, {v.note + v.offset, v.vel});
			end
		end
	end
end

return midi