local build = require("build")
local log = require("log")
local serialize = require("lib/serialize")

local save = {}

save.default_save_location = "out/project.sav"
save.last_save_location = save.default_save_location

local last_save = "out/last_save"

function save.set_save_location(filename)
	save.last_save_location = filename
	save.write_save_location()
end

function save.write_save_location()
	util.writefile(last_save, save.last_save_location)
end

function save.get_last_save_location()
	if util.file_exists(last_save) then
		save.last_save_location = util.readfile(last_save)
	else
		util.writefile(last_save, save.last_save_location)
	end

	return save.last_save_location
end

function save.write(filename)
	-- TODO: check overwrite
	log.info('saving project "' .. filename .. '"')

	local content = serialize(project, "project")
	util.writefile(filename, content)
	save.set_save_location(filename)
end

local function do_patches(p)
	-- fix any issues with save files here when they come up
	-- this is just a band-aid for now

	-- fix projects with different rank
	local tuning = require("tuning")
	for _, ch in ipairs(p.channels) do
		for _, note in ipairs(ch.notes) do
			for i = 1, tuning.rank do
				if not note.pitch[i] then
					note.pitch[i] = 0
				end
			end
		end
	end
end

function save.read(filename)
	if not util.file_exists(filename) then
		log.warn('could not find "' .. filename .. '"')

		return false
	end

	log.info('loading project "' .. filename .. '"')

	local content = util.readfile(filename)
	local new_project = setfenv(loadstring(content), {})()

	-- we will check versions, but only emit a warning
	local current_v = util.version_str(VERSION)
	local project_v = util.version_str(new_project.VERSION)
	if project_v ~= current_v then
		log.error(
			"Save file was created with version "
				.. project_v
				.. " which is incompatible with current version "
				.. current_v
		)
		return
	end

	-- if we can automatically upgrade save files to new version, do so here
	do_patches(new_project)

	build.load_project(new_project)
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
		setup = setfenv(loadstring(content), {})()
	else
		log.info("No settings found, generating default.")
		setup = {}
		setup.VERSION = util.clone(VERSION)
		setup.configs = {}
		setup.midi_devices = {}
	end

	return setup
end

return save
