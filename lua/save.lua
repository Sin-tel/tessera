local build = require("build")
local log = require("log")
local serialize = require("lib/serialize")

local save = {}

save.last_save_location = "out/project.sav"

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

local function do_patches()
	-- fix any issues with save files here when they come up
	-- this is just a band-aid for now

	-- fix projects with different rank
	local tuning = require("tuning")
	for _, ch in ipairs(project.channels) do
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

	-- currently assume most pessimistic compatibility
	-- later on we may be more lenient (probably should according to semver)
	-- if we can automatically upgrade save files to new version, do so here
	if
		new_project.VERSION.MAJOR == VERSION.MAJOR
		and new_project.VERSION.MINOR == VERSION.MINOR
		and new_project.VERSION.PATCH == VERSION.PATCH
	then
		project = new_project

		do_patches()
		build.project()
		return true
	end

	local save_version = new_project.VERSION.MAJOR
		.. "."
		.. new_project.VERSION.MINOR
		.. "."
		.. new_project.VERSION.PATCH
	log.warn("Save file version incompatible (" .. save_version .. ")")

	return false
end

local setup_path = "settings/setup.lua"

function save.write_setup()
	local content = serialize(setup, "setup")
	util.writefile(setup_path, content)
end

function save.read_setup()
	if util.file_exists(setup_path) then
		local content = util.readfile(setup_path)
		setup = setfenv(loadstring(content), {})()
	else
		-- build default setup and save it
		setup = {}
		setup.audio = {}
		setup.audio.default_host = "default"
		setup.audio.default_device = "default"
		setup.audio.buffer_size = 128
		setup.midi = {}
		setup.midi.inputs = { { name = "default" } }

		save.write_setup()
	end

	return setup
end

return save
