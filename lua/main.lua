release = false

local audiolib = require("audiolib")
local wav = require("./lib/wav_save")
local rtmidi = require("./lib/rtmidi_ffi")
local bit = require("bit")


io.stdout:setvbuf("no")

width = 720 --1280
height = 720

love.window.setMode( width, height, {vsync = false} )

blocks = {}

function love.load()
	-- put host and output device names here
	-- case insensitive, matches substrings
	print("========AUDIO========")
	audiolib.load("asio", "asio4all")
	-- audiolib.load("wasapi") 

	print("========MIDI========")
	indevice = rtmidi.createIn()
	rtmidi.printPorts(indevice)

	rtmidi.openPort(indevice, 2)
	-- rtmidi.openPort(indevice, 1)

	-- indevice2 = rtmidi.createIn()
	-- rtmidi.openPort(indevice2, 1)

	rtmidi.ignoreTypes(indevice, true, true, true)
end

numch = 1

note = 12
offset = 0
vel = 0

function printx(x)
  print("0x"..bit.tohex(x))
end

function love.update(dt)
	local index = 0


	while true do
		local msg, s = rtmidi.getMessage(indevice)
		if s == 0 then
			break
		end
		print(msg, s)

		local status = bit.rshift(msg.data[0], 4)
		local channel = bit.band(msg.data[0], 15)

		index = math.min(numch, channel) - 1		

		local b = msg.data[1]
		local c = 0
		if s > 2 then
			c = msg.data[2]
		end

		print(status, index)


		if status == 9 and c > 0 then -- note on
			vel = c/127
			note = b
		elseif status == 8 or (status == 9 and c == 0) then -- note off
			vel = 0
		elseif status == 13 then
			vel = b/127
		elseif status == 14 then
			offset = 48*(b+c*128 - 8192)/8192
			print(offset)
		end

		-- for i = 0, s-1 do
			-- print(msg.data[i])
		-- end
		-- rtmidi.sendMessage(outdevice, msg)
	end

	audiolib.send_CV(index, {note + offset, vel});


	audiolib.parse_messages()
end

function love.draw()
	mouseX, mouseY = love.mouse.getPosition()
	-- print(mouseY/44100)
	-- print(1/dt)
	love.graphics.ellipse("fill", (note+offset)*10, 200, vel*20)

	-- if not audiolib.paused then
	-- 	local index = 0
	-- 	-- print(index, numch)
	-- 	audiolib.send_CV(index, {note + offset, vel});
	-- end
	-- collectgarbage()
end

function love.keypressed( key, isrepeat )
	if key == 'escape' then
			love.event.quit()
	elseif key == 'q' then
		audiolib.quit()
	elseif key == 's' then
		audiolib.load()
	elseif key == 'p' then
		if audiolib.paused then
			audiolib.play()
		else
			audiolib.pause()
		end
	elseif key == 'r' then
		render_wav()
	elseif key == 'a' then
		-- for i = 1, 20 do
			numch = (numch or 1) + 1 
			print("numch: " .. numch)
			audiolib.add()
		-- end
	end

end

function love.quit()
	audiolib.quit()
end

function render_wav()
	audiolib.pause()

	-- sleep for a bit to make sure the audio thread is done
	love.timer.sleep(0.01)

	wav.open()
	for n = 1,5000 do
		local block = audiolib.render_block()
		local s = block.ptr
		local samples = {}
		for i = 1, tonumber(block.len) do
			samples[i] = s[i-1]
		end
		wav.append(samples)
	end
	wav.close()
	audiolib.play()
end