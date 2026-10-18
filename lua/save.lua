local build = require("build")
local log = require("log")
local serialize = require("lib/serialize")
local tuning_presets = require("default.tuning_presets")

local save = {}

local last_save = "out/last_save"

function save.set_save_location(filename)
	assert(filename)
	util.writefile(last_save, filename)
end

function save.get_save_location()
	if util.file_exists(last_save) then
		local filename = util.readfile(last_save)
		if filename and filename ~= "" then
			return filename
		end
	end
end

function save.write(filename)
	log.info('saving project "' .. filename .. '"')

	-- ensure all VST state is serialized to project
	for ch_index, channel in ipairs(project.channels) do
		if channel.instrument and channel.instrument.plugin then
			channel.instrument.plugin.state = tessera.audio.vst_get_state(ch_index)
		end
	end

	local content = serialize(project, "project")
	util.writefile(filename, content)
	save.set_save_location(filename)
end

save.recovery_path = "out/crash_recovery.sav"

-- Skips VST state (audio may be dead) and doesn't touch the save location.
function save.write_recovery()
	if not project or not project.channels then
		return false
	end
	log.info('saving crash recovery to "' .. save.recovery_path .. '"')
	util.writefile(save.recovery_path, serialize(project, "project"))
	return true
end

-- Patch any issues with save files from earlier versions.
function save.patch(p)
	local default = require("default.empty_project")()

	util.copy_defaults(p, default)

	-- 0.1.1 -> 0.1.2
	for _, ch in ipairs(p.channels) do
		-- add channel gain
		if not ch.gain then
			ch.gain = 1.0
		end
		-- rename pitch -> interval
		if ch.notes then
			for _, note in ipairs(ch.notes) do
				if not note.interval then
					note.interval = note.pitch
					note.pitch = nil
				end
			end
		end
	end

	-- 0.1.2 -> 0.1.3
	if #p.channels == 0 or not p.channels[1].master then
		local master_ch = build.new_channel_data({ master = true, name = "Master" })
		table.insert(p.channels, 1, master_ch)
	end

	-- 0.1.2 had no tuning settings, everything was meantone without extra accidentals.
	if not p.settings.tuning then
		p.settings.tuning = util.clone(tuning_presets.tunings.meantone)
		p.settings.tuning.notation = { accidentals = 0, half_sharp = false }
	end

	-- quick hack to get my own files working, should not go in release
	if not release and p.settings.tuning then
		local rank = p.settings.tuning.notation.accidentals + 2
		for _, ch in ipairs(p.channels) do
			if ch.notes then
				for _, note in ipairs(ch.notes) do
					for i = 1, rank do
						if not note.interval[i] then
							note.interval[i] = 0
						end
					end
				end
			end
		end
	end

	-- after patches are done, file should be on new version
	p.VERSION = VERSION
end

function save.read(filename)
	if not util.file_exists(filename) then
		log.warn('Could not find "' .. filename .. '"')
		return false
	end

	log.info('Loading project "' .. filename .. '"')

	local content = util.readfile(filename)
	local new_project = setfenv(loadstring(content), {})()

	-- we will check versions, but only emit a warning
	local current_v = util.version_str(VERSION)
	local project_v = util.version_str(new_project.VERSION)
	if not util.version_compatible(VERSION, new_project.VERSION) then
		log.error("Save file was created with " .. project_v .. " which is incompatible with " .. current_v)
		return
	end

	-- if we can automatically upgrade save files to new version, do so here
	save.patch(new_project)

	build.load_project(new_project)

	save.set_save_location(filename)
	return true
end

local setup_path = "out/setup.lua"

function save.write_setup()
	local content = serialize(setup, "setup")
	util.writefile(setup_path, content)
end

function save.read_setup()
	if util.file_exists(setup_path) then
		local content = util.readfile(setup_path)
		local new_setup = setfenv(loadstring(content), {})()

		local new_v = util.version_str(new_setup.VERSION)
		if util.version_compatible_strict(VERSION, new_setup.VERSION) then
			log.info("Loaded setup.lua")
			return new_setup
		else
			-- Losing the setup file is not really a big deal, so don't try to be clever about it
			log.warn("File setup.lua created with version " .. new_v .. ". Resetting.")
		end
	else
		log.info("No setup.lua found, generating default.")
	end

	local new_setup = {}
	new_setup.VERSION = util.clone(VERSION)
	new_setup.configs = {}
	new_setup.midi_devices = {}
	return new_setup
end

function save.init_setup()
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

function save.read_plugins()
	if not setup.plugin_list then
		save.scan_plugins()
	end
	return setup.plugin_list
end

local whitelist = { "pianoteq", "surge xt" }

function save.scan_plugins()
	setup.plugin_list = tessera.scan_plugins()

	for _, plugin in ipairs(setup.plugin_list) do
		for _, w in ipairs(whitelist) do
			if string.find(string.lower(plugin.name), w) then
				plugin.enabled = true
			end
		end
	end
end

return save
