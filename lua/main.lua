release = true

require("errorhandler")
require("run")

lurker = false

local settings_handler = require("settings_handler")
local audiolib = require("audiolib")
local wav = require("lib/wav_save")
local midi = require("midi")

if not release and lurker then
	lurker = require("lib/lurker")
end

require("util")
require("mouse")
require("ui")
require("views")
require("workspace")
require("parameter")

-- load color theme
require("settings/theme")
White = {1.0,1.0,1.0}


io.stdout:setvbuf("no")

width, height = love.graphics.getDimensions( )

audio_status = "wait"

function audioSetup()
	-- audiolib.load(settings.audio.default_host, settings.audio.default_device)
	audiolib.load("wasapi") 

	-- midi_in = midi.load(settings.midi.default_input)
	
	audiolib.add()

	audiolib.send_noteOn(0, {49   , 1.0});
	audio_status = "done"
end

function love.load()
	math.randomseed(os.time())
	love.math.setRandomSeed(os.time())
	settings = settings_handler.load()
	Mouse:load()

	-- font_main = love.graphics.newFont(12)
	font_main = love.graphics.newFont("res/dejavu_normal.fnt", "res/dejavu_normal.png")
	font_notes = love.graphics.newImageFont("res/font_notes.png",
		" ABCDEFGHIJKLMNOPQRSTUVWXYZ"..
		"0123456789.+-/"..
		"qwerty".. --flats/sharps  b#
		"asdfgh" .. -- pluses minuses  +-
		"zxcvbn" .. -- septimals L7
		"iopjkl" .. -- quarternotes / undecimals  dt
		"{[()]}" .. -- ups/downs  v^
		"!@#$&*"    -- arrows   ??
		,-1)

	love.graphics.setFont(font_main)

	gainp = Parameter:new("gain", {default = -12,  t = "dB"})
	panp = Parameter:new("pan", {default = 0, min = -1,max = 1, centered = true, fmt = "%0.2f"})

	Workspace:load()
	Workspace.box:split(0.7, true)
	Workspace.box.children[2]:split(0.7, false)

	Workspace.box.children[1]:setView(DefaultView:new())
	Workspace.box.children[2].children[2]:setView(PannerView:new())
	Workspace.box.children[2].children[1]:setView(ParameterView:new())
end


function love.update(dt) 
	-- print(1/dt)
	audiolib.parse_messages()

	audiolib.send_pan(0, {gainp.v, panp.v})

	-- midi.update(midi_in, handle_midi)
end


function love.draw()
	----update--------
	if audio_status == "request" then
		audioSetup()
	elseif audio_status == "wait" then
		audio_status = "request"
	end
	if not release and lurker then
		lurker.update()
	end

	Mouse:update()
	Workspace:update()


	Mouse:updateCursor()
	----draw----------
	love.graphics.clear()
	love.graphics.setColor(Theme.borders)
	love.graphics.rectangle("fill", 0,0, width, height)

	Workspace:draw()

	love.graphics.setColor(1.0, 1.0, 1.0)
	-- midi.draw(midi_in)

	love.graphics.setColor(1.0, 0.0, 0.0)
	for i,v in ipairs(tracks) do
		if v.isPlaying then
			love.graphics.ellipse("fill", (v.note)*10, 500, 10)
		end
	end
end

function love.mousepressed(x, y, button)
	Mouse:pressed(x, y, button)
end

function love.mousereleased(x, y, button)
	Mouse:released(x, y, button)
end

function love.wheelmoved( x, y )
	Workspace:wheelmoved(y)
end

-- function love.textinput(t)
--     print(t)
-- end

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
		for i = 1, 5 do
			numch = (numch or 0) 
			print("numch: " .. numch)
			audiolib.add()
			audiolib.send_noteOn(numch, {(35 + numch)%300  , 0.1});
			audiolib.send_pan(numch, {0.25, math.random()*2.0 - 1.0})
			numch = numch + 1
		end
	end

end

function love.resize(w, h)
	width = w
	height = h

	-- canvas = love.graphics.newCanvas(width, height)

	Workspace:resize(width,height)
end

function love.quit()
	settings_handler.save(settings)
	
	audiolib.quit()
end

function render_wav()
	love.mouse.setCursor( cursor_wait )

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
		-- print(s[1])
		wav.append(samples)
	end
	wav.close()
	audiolib.play()

	love.mouse.setCursor( cursor_default )
end