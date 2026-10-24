-- New and Open are undoable, and undo restores the file Save writes to.
-- cargo run -- --test-run tests/lua/open.lua

local build = require("build")
local file = require("file")
local save = require("save")

local h = dofile("tests/lua/lib/harness.lua")

-- no audio in test runs: every call is a no-op that returns a meter id
tessera.audio = setmetatable({}, {
	__index = function()
		return function()
			return 1
		end
	end,
})

local path_a = "out/test_open_a.sav"
local path_b = "out/test_open_b.sav"

-- opening writes it, put it back afterwards
local last_save = "out/last_save"
local prev_last_save = util.file_exists(last_save) and util.readfile(last_save)

-- a project named `name` belonging to `path`, with nothing on the undo stack
local function new_project(name, path)
	build.new_project()
	project.channels[1].name = name
	file.set_path(path)
	command.stack = {}
	command.index = 0
end

local function check(name, path, index)
	assert(project.channels[1].name == name, "project is " .. project.channels[1].name .. ", expected " .. name)
	assert(file.path == path, "path is " .. tostring(file.path))
	assert(command.index == index, "undo index is " .. command.index)
end

local tests = {}

function tests.open()
	new_project("B")
	save.write(path_b)
	new_project("A", path_a)

	file.open_path(path_b)
	check("B", path_b, 1)
	assert(util.readfile(last_save) == path_b)
	h.undo()
	check("A", path_a, 0)
	assert(util.readfile(last_save) == path_a)
	command.redo()
	check("B", path_b, 1)
end

function tests.failed_open_keeps_project()
	new_project("A", path_a)

	file.open_path("out/test_open_missing.sav")
	check("A", path_a, 0)
end

function tests.new()
	new_project("A", path_a)

	file.new()
	check("Master", nil, 1)
	h.undo()
	check("A", path_a, 0)
end

local failures = h.run(tests)

os.remove(path_b)
if prev_last_save then
	util.writefile(last_save, prev_last_save)
else
	os.remove(last_save)
end

return failures
