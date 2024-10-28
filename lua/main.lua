release = false

local log = require("log")
require("lib/run")

if not release then
	require("lib/strict")
end

local settingsHandler = require("settings_handler")
local backend = require("backend")
local midi = require("midi")
local views = require("views")
local command = require("command")
local save = require("save")

local VERSION = "0.0.1"

workspace = require("workspace")
mouse = require("mouse")
local note_input = require("note_input")
util = require("util")
channelHandler = require("channel_handler")

width, height = love.graphics.getDimensions()

time = 0

theme = require("settings/theme")
selection = {}
settings = {}
resources = {}

audio_status = "waiting"

project = {}
ui_channels = {}

local load_last_save = true
local last_save_location = "../out/lastsave.sav"

--- temp stuff, to delete ---

-----------------------------
local render
local sendParameters

local function audioSetup()
	if not backend:running() then
		-- backend:setup(settings.audio.default_host, settings.audio.default_device, settings.audio.buffer_size)
		backend:setup("wasapi", settings.audio.default_device)

		midi.load(settings.midi.inputs)
	else
		log.warn("Audio already set up")
	end

	if backend:running() then
		audio_status = "running"
	else
		log.error("Audio setup failed")
		audio_status = "dead"
	end

	if load_last_save and util.fileExists(last_save_location) then
		save.read(last_save_location)
	else
		local ch = channelHandler.newChannel("epiano")
		ch.armed = true
	end
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
	local top_left1, bottom_left = left:split(0.8, false)
	local top_left, middle_left = top_left1:split(0.3, false)
	local top_right, bottom_rigth = right:split(0.3, false)

	bottom_left:setView(views.TestPad:new())
	top_left:setView(views.Scope:new(false))
	middle_left:setView(views.Debug:new())
	-- middle_left:setView(views.UiTest:new())
	top_right:setView(views.Channels:new())
	bottom_rigth:setView(views.ChannelSettings:new())

	-- load empty project
	project.channels = {}
	ui_channels = {}
	project.VERSION = VERSION
	project.name = "Untitled project"
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
	mouse:update()
	backend:updateScope()
	workspace:update()

	mouse:endFrame()

	if backend:running() then
		sendParameters()
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

function love.keypressed(key, scancode, isrepeat)
	local ctrl = love.keyboard.isDown("lctrl", "rctrl")
	local shift = love.keyboard.isDown("lshift", "rshift")
	local alt = love.keyboard.isDown("lalt", "ralt")

	if not (ctrl or shift or alt) and note_input:keypressed(key, scancode, isrepeat) then
		return
	end

	if key == "escape" then
		love.event.quit()
	elseif key == "k" then
		if backend:running() then
			midi.quit()
			backend:quit()
		else
			audio_status = "request"
		end
	elseif key == "z" and ctrl then
		command.undo()
	elseif key == "y" and ctrl then
		command.redo()
	elseif key == "z" then
		log.info("(un)pausing backend")
		backend:setPaused(not backend:paused())
	elseif key == "r" and ctrl then
		render()
	elseif key == "s" and ctrl then
		save.write(last_save_location)
	elseif key == "a" and ctrl then
		channelHandler.newChannel("fm")
	elseif key == "down" and shift then
		if selection.channel_index and selection.device_index then
			channelHandler.reorderEffect(selection.channel_index, selection.device_index, 1)
		end
	elseif key == "up" and shift then
		if selection.channel_index and selection.device_index then
			channelHandler.reorderEffect(selection.channel_index, selection.device_index, -1)
		end
	elseif key == "delete" then
		if selection.channel_index then
			if selection.device_index then
				channelHandler.removeEffect(selection.channel_index, selection.device_index)
			else
				channelHandler.removeChannel(selection.channel_index)
				selection.channel_index = nil
			end
			selection.device_index = nil
		end
	end
end

function love.keyreleased(key, scancode)
	if note_input:keyreleased(key, scancode) then
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
	backend:quit()
end

function render()
	--TODO: make this not block the UI

	if not backend:running() then
		log.error("Backend offline.")
		return
	end

	log.info("Start render.")

	mouse:setCursor("wait")
	mouse:endFrame()

	backend:setPaused(true)

	-- sleep for a bit to make sure the audio thread is done
	love.timer.sleep(0.01)

	for _ = 1, 5000 do
		local success = backend:renderBlock()
		if not success then
			log.error("Failed to render block.")
			backend:play()
			return
		end
		parseMessages()
	end
	log.info("Finished render.")
	backend:renderFinish()

	backend:setPaused(false)

	mouse:setCursor("default")
end

local function toNumber(x)
	if type(x) == "number" then
		return x
	elseif type(x) == "boolean" then
		return x and 1 or 0
	else
		error("unsupported type: " .. type(x))
	end
end

function sendParameters()
	for k, ch in
		ipairs( --
			ui_channels
		)
	do
		for l, par in ipairs(ch.instrument.parameters) do
			local new_value = ch.instrument.state[l]
			local old_value = ch.instrument.state_old[l]
			if old_value ~= new_value then
				local value = toNumber(new_value)
				backend:sendParameter(k, 0, l, value)
				ch.instrument.state_old[l] = new_value
			end
		end

		for e, fx in ipairs(ch.effects) do
			for l, par in ipairs(fx.parameters) do
				local new_value = fx.state[l]
				local old_value = fx.state_old[l]
				if old_value ~= new_value then
					local value = toNumber(new_value)
					backend:sendParameter(k, e, l, value)
					fx.state_old[l] = new_value
				end
			end
		end
	end
end
