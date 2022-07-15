release = false

require("lib/errorhandler")
require("lib/run")

local lurker = true

local settingsHandler = require("settings_handler")
local audiolib = require("audiolib")
local wav = require("lib/wav_save")
local midi = require("midi")

if not release and lurker then
	lurker = require("lib/lurker")
end

require("util")
require("mouse")
require("keyboard")
require("ui")
require("views")
require("workspace")
require("parameter")
require("devicelist")
require("channel_handler")
require("pitch")

-- load color theme
require("settings/theme")

io.stdout:setvbuf("no")

width, height = love.graphics.getDimensions()

audio_status = "wait"

selection = {}

time = 0

-- temp stuff, to delete

local function audioSetup()
	audiolib.load(settings.audio.default_host, settings.audio.default_device)

	midi_in = midi.load(settings.midi.default_input)

	channelHandler:load()
	channelHandler:add("sine").armed = true
	-- for i = 1, 150 do
	-- 	local n = channelHandler:add("sine")
	-- 	n.parameters[2]:setNormalized(math.random())
	-- 	audiolib.send_noteOn(n.index, {math.random()*36+36, 0.5})
	-- end

	audio_status = "done"
end

function love.load()
	math.randomseed(os.time())
	love.math.setRandomSeed(os.time())
	settings = settingsHandler.load()
	mouse:load()

	---load resources---

	-- fonts.main = love.graphics.newFont(12)
	fonts = {}
	fonts.main = love.graphics.newFont("res/dejavu_normal.fnt", "res/dejavu_normal.png")
	fonts.notes = love.graphics.newImageFont(
		"res/font_notes.png",
		" ABCDEFGHIJKLMNOPQRSTUVWXYZ"
			.. "0123456789.+-/"
			.. "qwerty" -- flats/sharps  b#
			.. "asdfgh" -- pluses minuses  +-
			.. "zxcvbn" -- septimals L7
			.. "iopjkl" -- quarternotes / undecimals  dt
			.. "{[()]}" -- ups/downs  v^
			.. "!@#$&*", -- arrows   ??
		-1
	)

	love.graphics.setFont(fonts.main)

	icons = {}
	icons.solo = love.graphics.newImage("res/solo.png")
	icons.mute = love.graphics.newImage("res/mute.png")
	icons.armed = love.graphics.newImage("res/armed.png")
	icons.visible = love.graphics.newImage("res/visible.png")
	icons.invisible = love.graphics.newImage("res/invisible.png")
	icons.lock = love.graphics.newImage("res/lock.png")
	icons.unlock = love.graphics.newImage("res/unlock.png")

	---setup workspace---
	workspace:load()
	workspace.box:split(0.7, true)

	-- workspace.box.children[1]:setView(DefaultView:new())
	workspace.box.children[1]:split(0.7, false)
	workspace.box.children[1].children[1]:setView(songView:new())
	workspace.box.children[1].children[2]:setView(testPadView:new())

	workspace.box.children[2]:split(0.5, false)
	workspace.box.children[2].children[1]:setView(channelView:new())
	workspace.box.children[2].children[2]:setView(parameterView:new())
end

function love.update(dt)
	time = time + dt
	-- print(1/dt)
	if audio_status == "done" then
		audiolib.parse_messages()

		midi.update(midi_in, handle_midi)

		channelHandler:update()
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

	mouse:update()
	workspace:update()

	mouse:updateCursor()

	----draw----------
	love.graphics.clear()
	love.graphics.setColor(theme.borders)
	love.graphics.rectangle("fill", 0, 0, width, height)

	workspace:draw()

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
	mouse:pressed(x, y, button)
end

function love.mousereleased(x, y, button)
	mouse:released(x, y, button)
end

function love.mousemoved(x, y, dx, dy, istouch)
	mouse:mousemoved(x, y, dx, dy, istouch)
end

function love.wheelmoved(x, y)
	workspace:wheelmoved(y)
end

-- function love.focus(f)
-- 	if f then
-- 		print("Window is focused.")
-- 	else
-- 		print("Window is not focused.")
-- 	end
-- end

-- function love.textinput(t)
--     print(t)
-- end

function love.keypressed(key, isrepeat)
	if keyboard:keypressed(key, isrepeat) then
		return
	end

	if key == "escape" then
		love.event.quit()
	elseif key == "k" then
		midi.close(midi_in)
		audiolib.quit()
	elseif key == "l" then
		-- audiolib.load()
		audio_status = "wait"
	elseif key == "b" then
		if audiolib.paused then
			audiolib.play()
		else
			audiolib.pause()
		end
	elseif key == "s" then
		render_wav()
	elseif key == "a" then
		local n = channelHandler:add("sine")
		n.parameters[2]:setNormalized(math.random())
	end
end

function love.keyreleased(key)
	if keyboard:keyreleased(key) then
		return
	end
end

function love.resize(w, h)
	width = w
	height = h

	workspace:resize(width, height)
end

function love.quit()
	settingsHandler.save(settings)

	audiolib.quit()
end

function render_wav()
	--TODO: make this into a coroutine so we can yield from it

	mouse:setCursor("wait")

	audiolib.pause()

	-- sleep for a bit to make sure the audio thread is done
	love.timer.sleep(0.01)

	wav.open()
	for _ = 1, 5000 do
		local block = audiolib.render_block()
		if not block then
			print("failed to get block. try again")
			wav.close()
			audiolib.play()
			return
		end
		local s = block.ptr
		local samples = {}
		for i = 1, tonumber(block.len) do
			samples[i] = s[i - 1]
		end
		-- print(s[1])
		wav.append(samples)

		audiolib.parse_messages()
	end
	wav.close()
	audiolib.play()

	mouse:setCursor("default")
end
