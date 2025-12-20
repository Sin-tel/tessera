local build = require("build")
local log = require("log")
local serialize = require("lib/serialize")

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

	local content = serialize(project, "project")
	util.writefile(filename, content)
	save.set_save_location(filename)
end

local function do_patches(p)
	-- patch any issues with save files here when they come up

	-- 0.1.1 -> 0.1.2
	for _, ch in ipairs(p.channels) do
		if not ch.gain then
			ch.gain = 1.0
		end
		for _, note in ipairs(ch.notes) do
			note.interval = note.pitch
			note.pitch = nil
		end
	end

	-- fix projects with different rank
	local tuning = require("tuning")
	for _, ch in ipairs(p.channels) do
		for _, note in ipairs(ch.notes) do
			for i = 1, tuning.rank do
				if not note.interval[i] then
					note.interval[i] = 0
				end
			end
		end
	end
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
	do_patches(new_project)

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
		if util.version_compatible(VERSION, new_setup.VERSION) then
			log.info("Loaded setup.lua")
			return new_setup
		else
			-- Losing the setup file is not really a big deal, so don't try to be clever about it
			log.warn("File setup.lua created with version " .. new_v .. ". Ignoring.")
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

return save
