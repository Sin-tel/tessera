release = false

local audiolib = require("audiolib")
local wav = require("./lib/wav_save")
local midi = require("midi")


io.stdout:setvbuf("no")

width = 720 --1280
height = 720

love.window.setMode( width, height, {vsync = false} )

blocks = {}

function love.load()
	-- put host and output device names here
	-- case insensitive, matches substrings
	print("========AUDIO========")
	-- audiolib.load("asio", "asio4all")
	audiolib.load("wasapi") 

	print("========MIDI========")
	midi_in = midi.load(2)
	
end

numch = 1

pitch = 0
vel = 0

function love.update(dt)
	audiolib.parse_messages()

	local update = midi.update(midi_in)
	-- print(update)


	if update then
		local noteOn = false
		local pitch = 0
		local pitch_ = 0
		local vel = 0
		local w = 0
		for i,v in ipairs(midi_in.voices) do
			if v.noteOn then
				noteOn = true;
				v.noteOn = false;
			end

			pitch = pitch + (v.note + v.offset)*v.vel

			vel = math.max(v.vel, vel)
			w = w + v.vel
		end
		print(vel)
		if w > 0 then
			pitch = pitch / w
		end

		local vel_shaped = vel * vel;
		if noteOn then
			audiolib.send_noteOn(0, {pitch, vel_shaped});
		else
			audiolib.send_CV(0, {pitch, vel_shaped});
		end
		

		
	end
	

	


	
end

function love.draw()
	mouseX, mouseY = love.mouse.getPosition()

	midi.draw(midi_in)
	-- print(mouseY/44100)
	-- print(1/dt)
	-- love.graphics.ellipse("fill", (note+offset)*10, 200, vel*20)

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