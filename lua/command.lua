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

    project = build.newProject()

    ui_channels = {}
    -- clear selection
    selection.ch_index = nil
    selection.device_index = nil
end

function newProject:reverse()
    project = util.clone(self.prev)
    build.project()
end

command.newProject = newProject

--
local noteUpdate = {}
noteUpdate.__index = noteUpdate

function noteUpdate.new(prev_state, new_state)
    local self = setmetatable({}, noteUpdate)

    assert(prev_state)
    assert(new_state)

    -- keep ref to actual notes, state is just copies
    self.notes = {}
    for i, v in ipairs(new_state) do
        table.insert(self.notes, v)
    end

    self.prev_state = prev_state
    self.new_state = util.clone(new_state)

    return self
end

function noteUpdate:run()
    for i, v in ipairs(self.notes) do
        for key, value in pairs(v) do
            self.notes[i][key] = self.new_state[i][key]
        end
    end
end

function noteUpdate:reverse()
    for i, v in ipairs(self.notes) do
        for key, value in pairs(v) do
            self.notes[i][key] = self.prev_state[i][key]
        end
    end
end

command.noteUpdate = noteUpdate

--
local noteDelete = {}
noteDelete.__index = noteDelete

function noteDelete.new()
    local self = setmetatable({}, noteDelete)

    self.mask = {}
    self.notes = {}

    for ch_index, channel in ipairs(project.channels) do
        self.notes[ch_index] = {}
        if channel.visible and not channel.lock then
            for _, note in ipairs(channel.notes) do
                if selection.mask[note] then
                    self.mask[note] = true
                    table.insert(self.notes[ch_index], note)
                end
            end
        end
    end

    return self
end

function noteDelete:run()
    -- remove selected notes
    for _, channel in ipairs(project.channels) do
        for i = #channel.notes, 1, -1 do
            if self.mask[channel.notes[i]] then
                table.remove(channel.notes, i)
            end
        end
    end
end

function noteDelete:reverse()
    for ch_index in ipairs(self.notes) do
        for _, note in ipairs(self.notes[ch_index]) do
            table.insert(project.channels[ch_index].notes, note)
        end
    end
end

command.noteDelete = noteDelete

command.newChannel, command.removeChannel, command.newEffect, command.removeEffect, command.reorderEffect =
    unpack(require("command_channel"))

return command
