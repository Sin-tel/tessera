release = false

local log = require("log")
require("lib/run")

if not release then
	require("lib/strict")
end
VERSION = {}
VERSION.MAJOR = "0"
VERSION.MINOR = "0"
VERSION.PATCH = "1"

local backend = require("backend")
local engine = require("engine")
local midi = require("midi")
local views = require("views")
local save = require("save")
local note_input = require("note_input")
local build = require("build")

workspace = require("workspace")
mouse = require("mouse")
command = require("command")
util = require("util")

width, height = love.graphics.getDimensions()

theme = require("settings/theme")
selection = {}
setup = {}
resources = {}

audio_status = "init"

project = {}
ui_channels = {}

local load_last_save = true
local last_save_location = "../out/lastsave.sav"

-- predeclarations
local sendParameters

local function load_project()
	local success = false
	if load_last_save and util.fileExists(last_save_location) then
		success = save.read(last_save_location)
	end

	if not success then
		log.info("Loading default project")
		-- command.newChannel.new("analog"):run()
		command.newChannel.new("epiano"):run()
		-- command.newChannel.new("polysine"):run()
		project.channels[1].armed = true

		-- pitch = {base_pitch, start_time, velocity, verts}
		-- verts = list of {time, pitch_offset, pressure}

		local tuning = require("tuning")
		-- local note = {
		-- 	time = 0,
		-- 	pitch = tuning.fromMidi(60),
		-- 	vel = 0.6,
		-- 	verts = { { 0, 0, 0.2 }, { 0.5, -0.5, 0.6 }, { 1.0, 1.8, 0.5 }, { 1.5, -0.5, 0.5 }, { 2.0, 0.0, 0.1 } },
		-- }
		-- table.insert(project.channels[1].notes, note)`

		local note = { pitch = tuning.fromMidi(60), time = 0, vel = 0.6, verts = { { 0, 0, 0.5 }, { 0.5, 0.0, 0.5 } } }
		table.insert(project.channels[1].notes, note)

		note = { pitch = tuning.fromMidi(64), time = 0.1, vel = 0.6, verts = { { 0, 0, 0.5 }, { 0.5, 0.0, 0.5 } } }
		table.insert(project.channels[1].notes, note)

		note = { pitch = tuning.fromMidi(67), time = 0.2, vel = 0.6, verts = { { 0, 0, 0.5 }, { 0.5, 0.0, 0.5 } } }
		table.insert(project.channels[1].notes, note)

		-- for i = 0, 6 do
		-- 	local n = i - 3
		-- 	local p = { -4 * n, 7 * n }

		-- 	--tuning.fromMidi(60 + i)
		-- 	local note = {
		-- 		pitch = p,
		-- 		time = i,
		-- 		vel = 0.6,
		-- 		verts = { { 0, 0, 0.5 }, { 0.5, 0.0, 0.5 } },
		-- 	}
		-- 	table.insert(project.channels[1].notes, note)
		-- end
	end
end

local function audioSetup()
	if not backend:ok() then
		-- backend:setup(setup.audio.default_host, setup.audio.default_device, setup.audio.buffer_size)
		backend:setup("wasapi", "default")
		midi.load(setup.midi.inputs)
	else
		log.warn("Audio already set up")
	end

	if backend:ok() then
		audio_status = "running"
	else
		log.error("Audio setup failed")
		audio_status = "dead"
	end

	if project.needs_init then
		load_project()
		project.needs_init = false
	else
		-- restore backend
		ui_channels = {}
		build.project()
	end
end

function love.load()
	log.info("Tessera v" .. VERSION.MAJOR .. "." .. VERSION.MINOR .. "." .. VERSION.PATCH)
	math.randomseed(os.time())
	love.math.setRandomSeed(os.time())
	setup = save.readSetup()
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
	middle_left:setView(views.Song:new())
	-- middle_left:setView(views.Debug:new())
	top_right:setView(views.Channels:new())
	bottom_rigth:setView(views.ChannelSettings:new())

	-- load empty project
	project = build.newProject()
	project.needs_init = true
end

function love.update(dt)
	if not backend:ok() and (audio_status == "render" or audio_status == "running") then
		log.warn("Backend died.")
		audio_status = "dead"
	end
	if audio_status == "render" then
		engine.render()
	else
		midi.update()
		engine.update(dt)
	end
end

function love.draw()
	--- update ---
	if audio_status == "request" then
		audioSetup()
	elseif audio_status == "init" then
		audio_status = "request"
	end

	backend:updateScope()
	if audio_status ~= "render" then
		mouse:update()
		workspace:update()
		mouse:endFrame()

		if backend:ok() then
			sendParameters()
		end
	end

	--- draw ---
	love.graphics.clear()
	love.graphics.setColor(theme.borders)
	love.graphics.rectangle("fill", 0, 0, width, height)

	workspace:draw()

	if audio_status == "render" then
		love.graphics.setColor(0, 0, 0, 0.7)
		love.graphics.rectangle("fill", 0, 0, width, height)

		love.graphics.setColor(theme.background)
		love.graphics.rectangle("fill", width * 0.3, height * 0.5 - 16, width * 0.4, 32)
		love.graphics.setColor(theme.widget)
		local p = engine.render_progress / engine.render_end
		love.graphics.rectangle("fill", width * 0.3 + 4, height * 0.5 - 12, (width * 0.4 - 8) * p, 24)
	end
end

function love.mousepressed(x, y, button)
	if audio_status == "render" then
		return
	end
	mouse:pressed(x, y, button)
end

function love.mousereleased(x, y, button)
	if audio_status == "render" then
		return
	end
	mouse:released(x, y, button)
end

function love.mousemoved(x, y, dx, dy, istouch)
	if audio_status == "render" then
		return
	end
	mouse:mousemoved(x, y, dx, dy, istouch)
end

function love.wheelmoved(_, y)
	if audio_status == "render" then
		return
	end
	mouse:wheelmoved(y)
end

function love.textinput(t)
	if audio_status == "render" then
		return
	end
	-- should we handle love.textedited? (for IMEs)
	-- TODO: handle utf-8
	-- print(t)b
end

function love.keypressed(_, key)
	local mod = {}
	mod.ctrl = love.keyboard.isDown("lctrl", "rctrl")
	mod.shift = love.keyboard.isDown("lshift", "rshift")
	mod.alt = love.keyboard.isDown("lalt", "ralt")
	mod.any = mod.ctrl or mod.shift or mod.alt

	if audio_status == "render" then
		if (key == "c" and mod.ctrl) or key == "escape" then
			backend:renderCancel()
			engine.renderEnd()
		end

		return
	end

	if not mod.any and note_input:keypressed(key, mod) then
		return
	end

	if workspace:keypressed(key, mod) then
		return
	end

	if key == "escape" then
		love.event.quit()
	elseif key == "space" then
		if engine.playing then
			engine.stop()
		else
			engine.start()
		end
	elseif mod.ctrl and key == "k" then
		if backend:ok() then
			audio_status = "dead"
			midi.quit()
			backend:quit()
		else
			audio_status = "request"
		end
	elseif mod.ctrl and key == "w" then
		-- for testing panic recovery
		backend:panic()
	elseif key == "z" and mod.ctrl then
		command.undo()
	elseif key == "y" and mod.ctrl then
		command.redo()
	elseif key == "r" and mod.ctrl then
		engine.renderStart()
	elseif key == "n" and mod.ctrl then
		command.run_and_register(command.newProject.new())
	elseif key == "s" and mod.ctrl then
		save.write(last_save_location)
	elseif key == "down" and mod.shift then
		if selection.ch_index and selection.device_index then
			local new_index = selection.device_index + 1
			command.run_and_register(command.reorderEffect.new(selection.ch_index, selection.device_index, new_index))
		end
	elseif key == "up" and mod.shift then
		if selection.ch_index and selection.device_index then
			local new_index = selection.device_index - 1
			command.run_and_register(command.reorderEffect.new(selection.ch_index, selection.device_index, new_index))
		end
	elseif key == "delete" then
		if selection.ch_index then
			if selection.device_index and selection.device_index > 0 then
				command.run_and_register(command.removeEffect.new(selection.ch_index, selection.device_index))
			else
				command.run_and_register(command.removeChannel.new(selection.ch_index))
			end
		end
	end
end

function love.keyreleased(_, key)
	if audio_status == "render" then
		return
	end
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
	-- save.writeSetup()
	backend:quit()
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
	for k, ch in ipairs(ui_channels) do
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
