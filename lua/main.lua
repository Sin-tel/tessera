release = false
local lurker = false

io.stdout:setvbuf("no")

require("lib/errorhandler")
require("lib/run")
if not release then
	require("lib/strict")
end

util = require("util")
local settingsHandler = require("settings_handler")
local audiolib = require("audiolib")
local wav = require("lib/wav_save")
local midilib = require("midilib")
local views = require("views")

if not release and lurker then
	lurker = require("lib/lurker")
end

workspace = require("workspace")
mouse = require("mouse")
keyboard = require("keyboard")
util = require("util")
channelHandler = require("channel_handler")

width, height = love.graphics.getDimensions()

time = 0

theme = require("settings/theme")
selection = {}
settings = {}
resources = {}

--- temp stuff, to delete ---

local function audioSetup()
	audiolib.load(settings.audio.default_host, settings.audio.default_device)

	midilib.load(settings.midi.inputs)

	channelHandler:load()
	local ch = channelHandler:add("wavetable")
	ch.armed = true
	channelHandler:add_effect(ch, "gain")
	-- for i = 1, 150 do
	-- 	local n = channelHandler:add("sine")
	-- 	n.parameters[2]:setNormalized(math.random())
	-- 	audiolib.send_noteOn(n.index, {math.random()*36+36, 0.5})
	-- end
end

local function render_wav()
	--@todo: make this into a coroutine so we can yield from it

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
		wav.append(samples)

		audiolib.parse_messages()
	end
	wav.close()
	audiolib.play()

	mouse:setCursor("default")
end

function love.load()
	math.randomseed(os.time())
	love.math.setRandomSeed(os.time())
	settings = settingsHandler.load()
	mouse:load()

	--- load resources ---

	-- fonts.main = love.graphics.newFont(12)
	resources.fonts = {}
	resources.fonts.main = love.graphics.newFont("res/dejavu_normal.fnt", "res/dejavu_normal.png")
	resources.fonts.notes = love.graphics.newImageFont(
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

	love.graphics.setFont(resources.fonts.main)

	resources.icons = {}
	resources.icons.solo = love.graphics.newImage("res/solo.png")
	resources.icons.mute = love.graphics.newImage("res/mute.png")
	resources.icons.armed = love.graphics.newImage("res/armed.png")
	resources.icons.visible = love.graphics.newImage("res/visible.png")
	resources.icons.invisible = love.graphics.newImage("res/invisible.png")
	resources.icons.lock = love.graphics.newImage("res/lock.png")
	resources.icons.unlock = love.graphics.newImage("res/unlock.png")

	--- setup workspace ---
	workspace:load()
	workspace.box:split(0.7, true)

	workspace.box.children[1]:split(0.7, false)
	workspace.box.children[1].children[1]:setView(views.scope:new())
	workspace.box.children[1].children[2]:setView(views.testpad:new())

	workspace.box.children[2]:split(0.5, false)
	workspace.box.children[2].children[1]:setView(views.channel:new())
	workspace.box.children[2].children[2]:setView(views.parameter:new())
end

function love.update(dt)
	time = time + dt

	midilib.update()
	if audiolib.status == "running" then
		audiolib.parse_messages()

		channelHandler:update()
	end
end

function love.draw()
	--- update ---
	if audiolib.status == "request" then
		audioSetup()
	elseif audiolib.status == "wait" then
		audiolib.status = "request"
	end
	if not release and lurker then
		lurker.update()
	end

	mouse:update()
	workspace:update()
	mouse:updateCursor()

	--- draw ---
	love.graphics.clear()
	love.graphics.setColor(theme.borders)
	love.graphics.rectangle("fill", 0, 0, width, height)

	workspace:draw()
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
		if audiolib.status == "running" then
			midilib.quit()
			audiolib.quit()
		elseif audiolib.status == "offline" then
			audiolib.status = "request"
		end
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
		-- n.parameters[2]:setNormalized(math.random())
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
	-- settingsHandler.save(settings)

	midilib.quit()
	audiolib.quit()
end
