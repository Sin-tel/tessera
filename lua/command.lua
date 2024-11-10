local backend = require("backend")
local build = require("build")

local command = {}

command.maxSize = 50
command.stack = {}
command.index = 0

function command.run_and_register(c)
    c:run()
    command.register(c)
end

function command.register(c)
    command.index = command.index + 1

    -- remove irrelevant future commands
    for i = #command.stack, command.index, -1 do
        command.stack[i] = nil
    end

    command.stack[command.index] = c

    -- remove events exceeding max size
    if #command.stack > command.maxSize then
        table.remove(command.stack, 1)
        command.index = command.index - 1
    end
end

function command.undo()
    if command.index >= 1 then
        command.stack[command.index]:reverse()
        command.index = command.index - 1
    else
        command.index = 0
        print("nothing to undo!")
    end
end

function command.redo()
    command.index = command.index + 1
    if command.stack[command.index] then
        command.stack[command.index]:run()
    else
        command.index = #command.stack
        print("nothing to redo!")
    end
end

-- a change to some variable
local change = {}
change.__index = change

function change.new(target, key, value)
    local self = setmetatable({}, change)

    assert(type(target) == "table")
    self.target = target
    self.key = key
    self.value = value
    self.prev_value = target[key]

    return self
end

function change:run()
    self.target[self.key] = self.value
end

function change:reverse()
    self.target[self.key] = self.prev_value
end

command.change = change

--
local newProject = {}
newProject.__index = newProject

function newProject.new()
    local self = setmetatable({}, newProject)

    self.prev = util.clone(project)
    return self
end

function newProject:run()
    -- cleanup current project
    if project.channels then
        for i = #project.channels, 1, -1 do
            backend:removeChannel(i)
        end
    end

    -- init empty project
    project = {}
    project.channels = {}
    ui_channels = {}
    project.VERSION = {}
    project.VERSION.MAJOR = VERSION.MAJOR
    project.VERSION.MINOR = VERSION.MINOR
    project.VERSION.PATCH = VERSION.PATCH
    project.name = "Untitled project"

    -- clear selection
    selection.channel_index = nil
    selection.device_index = nil
end

function newProject:reverse()
    project = util.clone(self.prev)
    build.project()
end

command.newProject = newProject

command.newChannel, command.removeChannel, command.newEffect, command.removeEffect, command.reorderEffect =
    unpack(require("command_channel"))

return command
