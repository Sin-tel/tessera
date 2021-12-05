local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")

local midi = {}

local function device(handle, mpe,n)
	local voices = {}
	if mpe then
		for i = 1,16 do
			voices[i] = {vel = 0.0, note = 49, offset = 0.0, y = 0, noteOn = false}
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

	port_n = rtmidi.findPort(device_handle, name)

	rtmidi.openPort(device_handle, port_n)


	rtmidi.ignoreTypes(device_handle, true, true, true)

	return device(device_handle, true, port_n)
end

function midi.update(device)
	local update = false
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


		if status == 9 and c > 0 then -- note on
			device.voices[index].vel = c/127
			device.voices[index].note = b
			device.voices[index].noteOn = true
			update = true
		elseif status == 8 or (status == 9 and c == 0) then -- note off
			device.voices[index].vel = 0
			update = true
		elseif status == 13 then
			device.voices[index].vel = b/127
			update = true
		elseif status == 14 then
			device.voices[index].offset = 48*(b+c*128 - 8192)/8192
			update = true
		elseif status == 11 then
			if b == 74 then
				device.voices[index].y = c/127
			end
		end

	end

	return update
end

function midi.draw(device) 
	for i,v in ipairs(device.voices) do
		love.graphics.ellipse("fill", (v.note+v.offset)*10, 500-200*v.y, v.vel*20)
	end
end

return midi