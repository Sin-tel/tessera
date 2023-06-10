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

local function audioSetup()
	if not backend:running() then
		backend:setup(settings.audio.default_host, settings.audio.default_device)
	else
		print("Audio already set up")
	end

	if backend:running() then
		audio_status = "running"
	else
		print("Audio setup failed")
		return
	end

	midilib.load(settings.midi.inputs)

	channelHandler:load()
	local ch = channelHandler:add("wavetable")
	ch.armed = true

	-- for i = 1, 150 do
	-- 	local n = channelHandler:add("sine")
	-- 	n.parameters[2]:setNormalized(math.random())
	-- 	backend:send_note_on(n.index, {math.random()*36+36, 0.5})
	-- end
end

-- update UI with messages from backend
local function parse_messages()
	while true do
		local p = backend:rx_pop()
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

local function render_wav()
	--TODO: make this into a coroutine so we can yield from it

	mouse:setCursor("wait")

	backend:set_paused(true)

	-- sleep for a bit to make sure the audio thread is done
	love.timer.sleep(0.01)

	wav.open()
	for _ = 1, 5000 do
		local block = backend:render_block()
		if not block then
			print("failed to get block. try again")
			wav.close()
			backend:play()
			return
		end

		wav.append(block)

		parse_messages()
	end
	wav.close()
	backend:set_paused(false)

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
	if backend:running() then
		parse_messages()
		channelHandler:update()
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

	backend:update_scope()
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

function love.wheelmoved(_, y)
	workspace:wheelmoved(y)
end

-- function love.textinput(t)
-- 	print(t)
-- end

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
		backend:set_paused(not backend:paused())
	elseif key == "s" then
		render_wav()
	elseif key == "a" then
		channelHandler:add("sine")

		-- local n = channelHandler:add("sine")
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
	backend:quit()
end
