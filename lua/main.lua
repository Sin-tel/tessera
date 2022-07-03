release = false

require("lib/errorhandler")
require("lib/run")

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
require("devicelist")
require("channel_handler")

-- load color theme
require("settings/theme")
White = {1.0,1.0,1.0}


io.stdout:setvbuf("no")

width, height = love.graphics.getDimensions( )

audio_status = "wait"

selection = {}

time = 0

-- temp stuff, to delete


function audioSetup()
	audiolib.load(settings.audio.default_host, settings.audio.default_device)
	-- audiolib.load("wasapi") 

	midi_in = midi.load(settings.midi.default_input)

	channels.init()
	channels.add("sine").armed = true
	channels.add("sine")
	channels.add("sine")

	audio_status = "done"
end

function love.load()
	math.randomseed(os.time())
	love.math.setRandomSeed(os.time())
	settings = settings_handler.load()
	Mouse:load()

	---load resources---

	-- font_main = love.graphics.newFont(12)
	font_main = love.graphics.newFont("res/dejavu_normal.fnt", "res/dejavu_normal.png")
	font_notes = love.graphics.newImageFont("res/font_notes.png",
		" ABCDEFGHIJKLMNOPQRSTUVWXYZ"..
		"0123456789.+-/"..
		"qwerty" .. --flats/sharps  b#
		"asdfgh" .. -- pluses minuses  +-
		"zxcvbn" .. -- septimals L7
		"iopjkl" .. -- quarternotes / undecimals  dt
		"{[()]}" .. -- ups/downs  v^
		"!@#$&*"    -- arrows   ??
		,-1)

	love.graphics.setFont(font_main)

	icons = {}
	icons.solo = love.graphics.newImage("res/solo.png")
	icons.mute = love.graphics.newImage("res/mute.png")
	icons.armed = love.graphics.newImage("res/armed.png")
	icons.visible = love.graphics.newImage("res/visible.png")
	icons.invisible = love.graphics.newImage("res/invisible.png")
	icons.lock = love.graphics.newImage("res/lock.png")
	icons.unlock = love.graphics.newImage("res/unlock.png")

	---setup workspace---
	Workspace:load()
	Workspace.box:split(0.7, true)

	-- Workspace.box.children[1]:setView(DefaultView:new())
	Workspace.box.children[1]:split(0.7, false)
	Workspace.box.children[1].children[1]:setView(SongView:new())
	Workspace.box.children[1].children[2]:setView(TestPadView:new())

	Workspace.box.children[2]:split(0.5, false)
	Workspace.box.children[2].children[1]:setView(ChannelView:new())
	Workspace.box.children[2].children[2]:setView(ParameterView:new())
	
end


function love.update(dt) 
	time = time + dt
	-- print(1/dt)
	if audio_status == "done" then
		audiolib.parse_messages()
		
		midi.update(midi_in, handle_midi)

		channels.update()
	end
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

	-- love.graphics.setColor(1.0, 0.0, 0.0)
	-- for i,v in ipairs(tracks) do
	-- 	if v.isPlaying then
	-- 		love.graphics.ellipse("fill", (v.note)*10, 500, 10)
	-- 	end
	-- end
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
		midi.close(midi_in)
		audiolib.quit()
	elseif key == 's' then
		-- audiolib.load()
		audio_status = "wait"
	elseif key == 'p' then
		if audiolib.paused then
			audiolib.play()
		else
			audiolib.pause()
		end
	elseif key == 'r' then
		render_wav()
	elseif key == 'a' then
		channels.add("sine")
		-- l = #channels.list - 1
		-- audiolib.send_noteOn(l, {(35 + l)%300  , 0.5});
		-- audiolib.send_pan(l, {0.05, math.random()*2.0 - 1.0})
	end

end

function love.resize(w, h)
	width = w
	height = h

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

		audiolib.parse_messages()
	end
	wav.close()
	audiolib.play()

	love.mouse.setCursor( cursor_default )
end