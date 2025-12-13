release = tessera.is_release()

local log = require("log")

if not release then
	require("lib/strict")
end

local load_last_save = true

local profile = false
-- local profile = require("lib.profile")
-- local profile = require("lib.profile2")

VERSION = {}

util = require("util")

local build = require("build")
local engine = require("engine")
local file = require("file")
local midi = require("midi")
local note_input = require("note_input")
local save = require("save")
local tuning = require("tuning")

workspace = require("workspace")
mouse = require("mouse")
command = require("command")

width, height = tessera.graphics.get_dimensions()

theme = require("default/theme")
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

local project_initialized = false
local draw_time_s = 0

-- patch up set_color to work with tables
tessera.graphics.set_color = function(t)
	tessera.graphics.set_color_f(unpack(t))
end

local function init_setup()
	-- if setup is not properly configured, populate it with defaults
	local hosts = tessera.audio.get_hosts()
	for _, host in ipairs(hosts) do
		if not setup.configs[host] then
			setup.configs[host] = {}
		end
		if not setup.configs[host].device then
			setup.configs[host].device = tessera.audio.get_default_output_device(host)
		end
	end

	if not setup.host then
		setup.host = tessera.audio.get_default_host()
	end
end

local function build_startup_project()
	local success = false
	if load_last_save then
		local f = save.get_last_save_location()
		success = save.read(f)
	end

	if not success then
		log.info("Loading default project")
		command.NewChannel.new("epiano"):run()
		project.channels[1].armed = true

		save.set_save_location(save.default_save_location)
	end
end

local function audio_setup()
	if not tessera.audio.ok() then
		engine.setup_stream()
		midi.load()
		engine.reset_parameters()
	else
		-- this should probably never happen
		log.warn("Audio already set up")
	end
	audio_status = "running"

	if not tessera.audio.ok() then
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

function tessera.load(test_run)
	VERSION = tessera.version()

	log.info("Tessera v" .. util.version_str(VERSION))
	if release then
		log.info("Running in release mode")
	else
		log.info("Running in debug mode")
	end

	math.randomseed(os.time())

	setup = save.read_setup()
	init_setup()

	tuning.load()

	mouse:load()

	if not test_run then
		-- setup workspace
		workspace:load()

		-- set up empty project
		build.new_project()
	end
end

function tessera.update(dt)
	-- protect against huge dt from frozen window
	dt = math.min(dt, 1 / 60)

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

	file.poll_dialogs()

	local t_start = tessera.get_time()

	tessera.audio.update_scope()
	engine.update_meters()

	if audio_status ~= "render" then
		mouse:update()
		workspace:update()
		engine.update_frame()
		mouse:end_frame()

		engine.send_parameters()
	end

	--- draw ---
	tessera.graphics.set_color(theme.borders)
	tessera.graphics.rectangle("fill", 0, 0, width, height)

	workspace:draw()

	-- status bar
	local draw_time = (tessera.get_time() - t_start) * 1000
	draw_time_s = draw_time_s + 0.1 * (draw_time - draw_time_s)
	local draw_time_l = string.format("%04.1f", draw_time_s)
	tessera.graphics.set_font_size(12)
	tessera.graphics.set_color(theme.text_tip)
	tessera.graphics.text(draw_time_l, 4, height - 15)
	tessera.graphics.text("v" .. util.version_str(VERSION), width - 48, height - 15)

	-- if modifier_keys.alt then
	-- 	tessera.graphics.draw_debug_atlas()
	-- end

	if audio_status == "render" then
		tessera.graphics.set_color_f(0, 0, 0, 0.2)
		tessera.graphics.rectangle("fill", 0, 0, width, height)

		tessera.graphics.set_color(theme.background)
		tessera.graphics.rectangle("fill", width * 0.3, height * 0.5 - 16, width * 0.4, 32)
		tessera.graphics.set_color(theme.widget)
		local p = engine.get_progress_relative()
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

function tessera.keypressed(key, key_str, isrepeat)
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
	elseif modifier_keys.ctrl and key == "tab" then
		workspace:switch_tab(modifier_keys.shift)
	elseif modifier_keys.ctrl and key == "t" then
		util.pprint(workspace:to_data())
	elseif modifier_keys.ctrl and key == "f" then
		engine.stop()
		tessera.audio.clear_messages()
		tessera.audio.flush()
	elseif modifier_keys.ctrl and key == "k" then
		tessera.audio.quit()
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
		file.new()
	elseif key == "o" and modifier_keys.ctrl then
		file.open()
	elseif key == "s" and modifier_keys.ctrl and modifier_keys.shift then
		file.save_as()
	elseif key == "s" and modifier_keys.ctrl then
		file.save()
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

function tessera.keyreleased(key, key_str)
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
	save.write_setup()
	log.info("Quitting")
end

function tessera.draw_error(msg)
	-- TODO: we can save project here, and try to restore on next run

	tessera.graphics.set_color(theme.borders)
	tessera.graphics.rectangle("fill", 0, 0, width, height)

	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", 16, 16, width - 32, height - 32)

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.set_font_size(20)
	tessera.graphics.set_font_main()

	local x, y = 80, 80
	for line in msg:gmatch("([^\n]*)\n?") do
		y = y + 24
		tessera.graphics.text(line, x, y)
	end

	y = y + 10
	tessera.graphics.set_font_size(16)
	tessera.graphics.text("Press escape to exit.", x, y)
end
