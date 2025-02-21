local serialize = require("lib/serialize")
local log = require("log")
local build = require("build")

local save = {}

function save.write(filename)
	log.info('saving project "' .. filename .. '"')

	local content = serialize(project, "project")
	util.writefile(filename, content)
end

function save.read(filename)
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

local setup_path = love.filesystem.getSource() .. "/settings/setup.lua"

function save.writeSetup()
	local content = serialize(setup, "setup")
	util.writefile(setup_path, content)
end

function save.readSetup()
	if util.fileExists(setup_path) then
		local content = util.readfile(setup_path)
		setup = setfenv(loadstring(content), {})()
	else
		setup = {}
		setup.audio = {}
		setup.audio.default_host = "default"
		setup.audio.default_device = "default"
		setup.audio.buffer_size = 128
		setup.midi = {}
		setup.midi.inputs = { { name = "default" } }

		save.writeSetup()
	end

	return setup
end

return save
