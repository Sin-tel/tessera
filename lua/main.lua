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
local midi = require("midi")
local views = require("views")

if not release and lurker then
	lurker = require("lib/lurker")
end

workspace = require("workspace")
mouse = require("mouse")
note_input = require("note_input")
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

	midi.load(settings.midi.inputs)

	channelHandler:load()
	-- local ch = channelHandler:add("sine")
	-- local ch = channelHandler:add("polysine")
	-- local ch = channelHandler:add("analog")
	local ch = channelHandler:add("fm")
	-- local ch = channelHandler:add("wavetable")

	channelHandler:addEffect(ch, "reverb")

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
	resources = require("resources")

	--- setup workspace ---
	workspace:load()
	local left, right = workspace.box:split(0.7, true)
	local top_left, bottom_left = left:split(0.8, false)
	local top_left, middle_left = top_left:split(0.3, false)
	local top_rigth, bottom_rigth = right:split(0.3, false)

	bottom_left:setView(views.TestPad:new())

	top_left:setView(views.Scope:new(false))
	middle_left:setView(views.Scope:new(true))
	-- middle_left:setView(views.UiTest:new())

	top_rigth:setView(views.Channels:new())
	bottom_rigth:setView(views.ChannelSettings:new())
end

function love.update(dt)
	time = time + dt

	midi.update()
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
	if note_input:keypressed(key, isrepeat) then
		return
	end

	local ctrl = love.keyboard.isDown("lctrl", "rctrl")
	local shift = love.keyboard.isDown("lshift", "rshift")
	local alt = love.keyboard.isDown("lalt", "ralt")

	if key == "escape" then
		love.event.quit()
	elseif key == "k" then
		if backend:running() then
			midi.quit()
			backend:quit()
		else
			audio_status = "request"
		end
	elseif key == "p" then
		backend:setPaused(not backend:paused())
	elseif key == "s" and ctrl then
		renderWav()
	elseif key == "a" and ctrl then
		channelHandler:add("fm")
	elseif key == "down" and shift then
		local ch = selection.channel
		local d = selection.device
		if ch and d then
			channelHandler:reorderEffect(ch, d, 1)
		end
	elseif key == "up" and shift then
		local ch = selection.channel
		local device = selection.device
		if ch and device then
			channelHandler:reorderEffect(ch, device, -1)
		end
	elseif key == "delete" then
		local ch = selection.channel
		if ch then
			if selection.device then
				channelHandler:removeEffect(ch, selection.device)
			else
				channelHandler:remove(ch)
				selection.channel = nil
			end
			selection.device = nil
		end
	end
end

function love.keyreleased(key)
	if note_input:keyreleased(key) then
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

	midi.quit()
	backend:quit()
end
