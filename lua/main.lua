release = false

audiolib = require("audiolib")

local wav = require("./lib/wav_save")

io.stdout:setvbuf("no")

width = 720 --1280
height = 720

love.window.setMode( width, height, {vsync = false} )

blocks = {}

function love.load()
	audiolib.load()
end

numch = 1

function love.update(dt)
	mouseX, mouseY = love.mouse.getPosition()
	-- print(mouseY/44100)
	-- print(1/dt)

	if not audiolib.paused then
		local index = math.random(numch) -1
		-- print(index, numch)
		audiolib.send_CV(index, {mouseX/44100 + love.math.randomNormal(0.001), 0.5*(1 - mouseY /height) / math.sqrt(numch)});
	end
	-- collectgarbage()
end

function love.draw()
	-- local x = 0
	-- for i in ipairs(blocks) do
	-- 	local block = blocks[i]
	
	-- 	samples = block.ptr
	-- 	for j = 0, tonumber(block.len)-1 do
	-- 		love.graphics.points(x,300+100*samples[j].l)
	-- 		x = x +1
	-- 	end
	-- end
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
		for i = 1, 20 do
			numch = (numch or 1) + 1 
			print("numch: " .. numch)
			audiolib.add()
		end
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