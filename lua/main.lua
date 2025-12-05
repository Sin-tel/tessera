release = tessera.audio.is_release()

local log = require("log")

if not release then
	require("lib/strict")
end

local profile = false
-- local profile = require("lib.profile")
-- local profile = require("lib.profile2")

VERSION = {}
VERSION.MAJOR = 0
VERSION.MINOR = 0
VERSION.PATCH = 1

util = require("util")

local build = require("build")
local engine = require("engine")
local midi = require("midi")
local note_input = require("note_input")
local save = require("save")
local views = require("views")

workspace = require("workspace")
mouse = require("mouse")
command = require("command")

width, height = tessera.graphics.get_dimensions()

theme = require("settings/theme")
selection = require("selection")
clipboard = require("clipboard")
setup = {}

audio_status = "init"

project = {}
ui_channels = {}

modifier_keys = {}
modifier_keys.ctrl = false
modifier_keys.shift = false
modifier_keys.alt = false
modifier_keys.any = false

local load_last_save = true
local project_initialized = false

local draw_time_s = 0

local dialog_pending

-- patch up set_color to work with tables
tessera.graphics.set_color = function(t)
	tessera.graphics.set_color_f(unpack(t))
end

local function build_startup_project()
	local success = false
	if load_last_save then
		local f = save.get_last_save_location()
		success = save.read(f)
	end

	if not success then
		log.info("Loading default project")
		command.NewChannel.new("pluck"):run()
		-- command.NewChannel.new("epiano"):run()
		-- command.NewChannel.new("polysine"):run()
		project.channels[1].armed = true
	end
end

local function audio_setup()
	if not tessera.audio.ok() then
		tessera.audio.setup(setup.audio.default_host, setup.audio.default_device, setup.audio.buffer_size)
		midi.load()
		engine.reset_parameters()
	else
		log.warn("Audio already set up")
	end

	if tessera.audio.ok() then
		audio_status = "running"
	else
		log.error("Audio setup failed")
	end

	if not project_initialized then
		build_startup_project()
		project_initialized = true
	else
		-- restore audio state
		build.restore_project()
	end
end

-- since file dialogs are spawned on new threads, we need to check the results here
local function poll_dialogs()
	if dialog_pending then
		local f = tessera.dialog_poll()
		if f then
			if dialog_pending == "save" then
				save.write(f)
				save.set_save_location(f)
				dialog_pending = nil
			elseif dialog_pending == "open" then
				-- TODO: undo
				build.new_project()
				save.read(f)
				save.set_save_location(f)
				dialog_pending = nil
			end
		end
	end
end

function tessera.load()
	log.info("Tessera v" .. VERSION.MAJOR .. "." .. VERSION.MINOR .. "." .. VERSION.PATCH)
	if release then
		log.info("Running in release mode")
	else
		log.info("Running in debug mode")
	end

	math.randomseed(os.time())

	setup = save.read_setup()

	mouse:load()

	--- setup workspace ---
	workspace:load()
	local left, right = workspace.box:split(0.7, true)
	local top_left, middle_left = left:split(0.2, false)
	local top_right, bottom_rigth = right:split(0.35, false)

	top_left:set_view(views.Scope.new(false))
	-- top_left:set_view(views.Canvas.new())
	middle_left:set_view(views.Canvas.new())
	-- middle_left:set_view(views.Debug.new())
	top_right:set_view(views.Channels.new())
	bottom_rigth:set_view(views.ChannelSettings.new())

	-- load empty project
	build.new_project()
end

function tessera.update(dt)
	-- protect against huge dt from frozen window
	dt = math.min(dt, 1 / 60)

	if tessera.audio.check_should_rebuild() then
		log.info("Rebuilding stream")
		tessera.audio.rebuild(setup.audio.default_host, setup.audio.default_device, setup.audio.buffer_size)
	end

	if audio_status == "render" then
		engine.render()
	elseif audio_status == "running" then
		midi.update(dt)
		engine.update(dt)
	end
end

function tessera.draw()
	--- update ---
	if audio_status == "request" then
		audio_setup()
	elseif audio_status == "init" then
		audio_status = "request"
	end

	if profile then
		profile.start()
	end

	poll_dialogs()

	local t_start = tessera.get_time()

	tessera.audio.update_scope()
	if audio_status ~= "render" then
		mouse:update()
		workspace:update()
		mouse:end_frame()

		engine.send_parameters()
	end

	--- draw ---
	tessera.graphics.set_color(theme.borders)
	tessera.graphics.rectangle("fill", 0, 0, width, height)

	workspace:draw()

	local draw_time = (tessera.get_time() - t_start) * 1000
	draw_time_s = draw_time_s + 0.1 * (draw_time - draw_time_s)
	local draw_time_l = string.format("%04.1f", draw_time_s)
	tessera.graphics.set_font_size(12)
	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(draw_time_l, 10, 0, 100, 32)

	if audio_status == "render" then
		tessera.graphics.set_color_f(0, 0, 0, 0.2)
		tessera.graphics.rectangle("fill", 0, 0, width, height)

		tessera.graphics.set_color(theme.background)
		tessera.graphics.rectangle("fill", width * 0.3, height * 0.5 - 16, width * 0.4, 32)
		tessera.graphics.set_color(theme.widget)
		local p = engine.render_progress / engine.render_end
		tessera.graphics.rectangle("fill", width * 0.3 + 4, height * 0.5 - 12, (width * 0.4 - 8) * p, 24)
	end

	if profile then
		profile.stop()
	end
end

function tessera.mousepressed(x, y, button)
	if audio_status == "render" then
		return
	end
	mouse:pressed(x, y, button)
end

function tessera.mousereleased(x, y, button)
	if audio_status == "render" then
		return
	end
	mouse:released(x, y, button)
end

function tessera.mousemoved(x, y, dx, dy)
	if audio_status == "render" then
		return
	end
	mouse:mousemoved(x, y, dx, dy)
end

function tessera.wheelmoved(_, y)
	if audio_status == "render" then
		return
	end
	mouse:wheelmoved(y)
end

function tessera.textinput(t)
	if audio_status == "render" then
		return
	end
	-- not implemented
end

function tessera.keypressed(_, key, isrepeat)
	if key == "lshift" or key == "rshift" then
		modifier_keys.shift = true
	elseif key == "lctrl" or key == "rctrl" then
		modifier_keys.ctrl = true
	elseif key == "lalt" or key == "ralt" then
		modifier_keys.alt = true
	end
	modifier_keys.any = modifier_keys.ctrl or modifier_keys.shift or modifier_keys.alt

	if audio_status == "render" then
		if (key == "c" and modifier_keys.ctrl) or key == "escape" then
			tessera.audio.render_cancel()
			engine.render_finish()
		end

		return
	end

	if not isrepeat and not modifier_keys.any and note_input:keypressed(key) then
		return
	end

	if workspace:keypressed(key) then
		return
	end

	if key == "escape" then
		tessera.exit()
	elseif key == "space" then
		if engine.playing then
			engine.stop()
		else
			engine.start()
		end
	-- elseif modifier_keys.ctrl and key == "t" then
	-- 	local t_start = tessera.get_time()
	-- 	log.info("Sending project to backend")
	-- 	tessera.project.set(project)
	-- 	local time = (tessera.get_time() - t_start) * 1000
	-- 	print("Took " .. time)
	-- elseif modifier_keys.ctrl and key == "r" then
	-- 	local p = tessera.project.get()
	-- 	if p then
	-- 		build.load_project(p)
	-- 	end
	elseif modifier_keys.ctrl and key == "f" then
		engine.stop()
		tessera.audio.clear_messages()
		tessera.audio.flush()
	elseif modifier_keys.ctrl and key == "k" then
		if tessera.audio.ok() then
			midi.quit()
			tessera.audio.quit()
		else
			audio_status = "request"
		end
	elseif modifier_keys.ctrl and key == "w" then
		-- for testing panic recovery
		tessera.audio.panic()
	elseif modifier_keys.ctrl and key == "p" then
		if profile then
			log.info(profile.report(20))
		end
	elseif key == "z" and modifier_keys.ctrl then
		command.undo()
	elseif key == "y" and modifier_keys.ctrl then
		command.redo()
	elseif key == "r" and modifier_keys.ctrl then
		engine.render_start()
	elseif key == "n" and modifier_keys.ctrl then
		command.run_and_register(command.NewProject.new())
	elseif key == "o" and modifier_keys.ctrl then
		if tessera.dialog_open() then
			dialog_pending = "open"
		end
	elseif key == "s" and modifier_keys.ctrl and modifier_keys.shift then
		if tessera.dialog_save("my_project") then
			dialog_pending = "save"
		end
	elseif key == "s" and modifier_keys.ctrl then
		save.write(save.last_save_location)
	elseif key == "b" then
		project.transport.recording = not project.transport.recording
	elseif key == "down" and modifier_keys.shift then
		if selection.ch_index and selection.device_index then
			local new_index = selection.device_index + 1
			command.run_and_register(command.ReorderEffect.new(selection.ch_index, selection.device_index, new_index))
		end
	elseif key == "up" and modifier_keys.shift then
		if selection.ch_index and selection.device_index then
			local new_index = selection.device_index - 1
			command.run_and_register(command.ReorderEffect.new(selection.ch_index, selection.device_index, new_index))
		end
	elseif key == "delete" then
		-- TODO: move these to respective views
		if selection.ch_index then
			if selection.device_index and selection.device_index > 0 then
				command.run_and_register(command.RemoveEffect.new(selection.ch_index, selection.device_index))
			else
				command.run_and_register(command.RemoveChannel.new(selection.ch_index))
			end
		end
	end
end

function tessera.keyreleased(_, key)
	if key == "lshift" or key == "rshift" then
		modifier_keys.shift = false
	elseif key == "lctrl" or key == "rctrl" then
		modifier_keys.ctrl = false
	elseif key == "lalt" or key == "ralt" then
		modifier_keys.alt = false
	end
	modifier_keys.any = modifier_keys.ctrl or modifier_keys.shift or modifier_keys.alt

	if audio_status == "render" then
		return
	end
	if note_input:keyreleased(key) then
		return
	end
end

function tessera.resize(w, h)
	width = w
	height = h

	workspace:resize(width, height)
end

function tessera.quit()
	-- save.writeSetup()
end
