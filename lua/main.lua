release = false
local lurker = false

io.stdout:setvbuf("no")

require("lib/run")

if not release then
	require("lib/strict")
end

local settingsHandler = require("settings_handler")
local backend = require("backend")
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

audio_status = "waiting"

--- temp stuff, to delete ---

-----------------------------

local function audioSetup()
	if not backend:running() then
		-- backend:setup(settings.audio.default_host, settings.audio.default_device)
		backend:setup("wasapi", settings.audio.default_device)
		-- audio_status = "running"
	else
		print("Audio already set up")
	end

	if backend:running() then
		audio_status = "running"
	else
		print("Audio setup failed")
	end

	midilib.load(settings.midi.inputs)

	channelHandler:load()
	-- local ch = channelHandler:add("sine")
	-- local ch = channelHandler:add("analog")
	local ch = channelHandler:add("fm")
	-- local ch = channelHandler:add("wavetable")

	-- channelHandler:addEffect(ch, "drive")
	ch.armed = true
end

-- update UI with messages from backend
local function parseMessages()
	while true do
		local p = backend:pop()
		if p == nil then
			return
		end
		if p.tag == "cpu" then
			workspace.cpu_load = p.cpu_load
		elseif p.tag == "meter" then
			workspace.meter.l = util.to_dB(p.l)
			workspace.meter.r = util.to_dB(p.r)
		end
	end
end

local function renderWav()
	--TODO: run this on a new thread in rust?

	if not backend:running() then
		print("backend offline")
		return
	end

	mouse:setCursor("wait")
	mouse:endFrame()

	backend:setPaused(true)

	-- sleep for a bit to make sure the audio thread is done
	love.timer.sleep(0.01)

	wav.open()
	for _ = 1, 5000 do
		local block = backend:renderBlock()
		if not block then
			print("failed to get block. try again")
			wav.close()
			backend:play()
			return
		end

		wav.append(block)

		parseMessages()
	end
	wav.close()
	backend:setPaused(false)

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
	local left, right = workspace.box:split(0.7, true)
	local top_left, bottom_left = left:split(0.8, false)
	local top_left, middle_left = top_left:split(0.3, false)
	local top_rigth, bottom_rigth = right:split(0.3, false)

	bottom_left:setView(views.TestPad:new())

	top_left:setView(views.Scope:new(false))
	middle_left:setView(views.Scope:new(true))

	top_rigth:setView(views.Channels:new())
	bottom_rigth:setView(views.ChannelSettings:new())
end

function love.update(dt)
	time = time + dt

	midilib.update()
	if backend:running() then
		parseMessages()
	end
end

function love.draw()
	--- update ---
	if audio_status == "request" then
		audioSetup()
	elseif audio_status == "waiting" then
		audio_status = "request"
	end
	if not release and lurker then
		lurker.update()
	end
	mouse:update()
	backend:updateScope()
	workspace:update()

	mouse:endFrame()

	if backend:running() then
		channelHandler:sendParameters()
	end

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

function love.wheelmoved(_, y)
	mouse:wheelmoved(y)
end

function love.textinput(t)
	-- should we handle love.textedited? (for IMEs)
	-- TODO: handle utf-8
	-- print(t)b
end

function love.keypressed(key, isrepeat)
	if keyboard:keypressed(key, isrepeat) then
		return
	end

	if key == "escape" then
		love.event.quit()
	elseif key == "k" then
		if backend:running() then
			midilib.quit()
			backend:quit()
		else
			audio_status = "request"
		end
	elseif key == "b" then
		backend:setPaused(not backend:paused())
	elseif key == "s" then
		renderWav()
	elseif key == "a" then
		-- channelHandler:add("sine")
		-- channelHandler:add("wavetable")
		-- channelHandler:add("analog")
		channelHandler:add("fm")
		-- print(#channelHandler.list)
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
	backend:quit()
end
