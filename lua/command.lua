local build = require("build")
local load_default_project = require("default.project")

local command = {}

command.max_size = 50
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
    if #command.stack > command.max_size then
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
local Change = {}
Change.__index = Change

function Change.new(target, key, value)
    local self = setmetatable({}, Change)

    assert(type(target) == "table")
    assert(value)
    self.target = target
    self.key = key
    self.value = value
    self.prev_value = target[key]

    return self
end

function Change:run()
    self.target[self.key] = self.value
end

function Change:reverse()
    self.target[self.key] = self.prev_value
end

command.Change = Change

--
local NewProject = {}
NewProject.__index = NewProject

function NewProject.new()
    local self = setmetatable({}, NewProject)

    self.prev = util.clone(project)
    return self
end

function NewProject:run()
    -- cleanup current project
    build.new_project()
    load_default_project()
end

function NewProject:reverse()
    build.load_project(util.clone(self.prev))
end

command.NewProject = NewProject

--
local NoteUpdate = {}
NoteUpdate.__index = NoteUpdate

function NoteUpdate.new(prev_state, new_state)
    local self = setmetatable({}, NoteUpdate)

    assert(prev_state)
    assert(new_state)

    -- keep ref to actual notes, state is just copies
    self.notes = {}
    for _, v in ipairs(new_state) do
        table.insert(self.notes, v)
    end

    self.prev_state = prev_state
    self.new_state = util.clone(new_state)

    return self
end

function NoteUpdate:run()
    for i, v in ipairs(self.notes) do
        for key in pairs(v) do
            self.notes[i][key] = self.new_state[i][key]
        end
    end
end

function NoteUpdate:reverse()
    for i, v in ipairs(self.notes) do
        for key in pairs(v) do
            self.notes[i][key] = self.prev_state[i][key]
        end
    end
end

command.NoteUpdate = NoteUpdate

--
local NoteDelete = {}
NoteDelete.__index = NoteDelete

function NoteDelete.new(notes)
    local self = setmetatable({}, NoteDelete)

    assert(notes)
    self.notes = notes
    self.mask = {}

    for ch_index in ipairs(self.notes) do
        for _, note in ipairs(self.notes[ch_index]) do
            self.mask[note] = true
        end
    end
    return self
end

function NoteDelete:run()
    -- remove selected notes
    for _, channel in ipairs(project.channels) do
        for i = #channel.notes, 1, -1 do
            if self.mask[channel.notes[i]] then
                table.remove(channel.notes, i)
            end
        end
    end
end

function NoteDelete:reverse()
    for ch_index in ipairs(self.notes) do
        for _, note in ipairs(self.notes[ch_index]) do
            table.insert(project.channels[ch_index].notes, note)
        end
    end
end

command.NoteDelete = NoteDelete

--
local NoteAdd = {}
NoteAdd.__index = NoteAdd

function NoteAdd.new(notes)
    local self = setmetatable({}, NoteAdd)

    self.notes = notes
    self.mask = {}

    -- use pairs because notes may be sparse
    for ch_index in pairs(self.notes) do
        for _, note in ipairs(self.notes[ch_index]) do
            self.mask[note] = true
        end
    end
    return self
end

function NoteAdd:run()
    for ch_index in pairs(self.notes) do
        for _, note in ipairs(self.notes[ch_index]) do
            table.insert(project.channels[ch_index].notes, note)
        end
    end
end

function NoteAdd:reverse()
    for _, channel in ipairs(project.channels) do
        for i = #channel.notes, 1, -1 do
            if self.mask[channel.notes[i]] then
                table.remove(channel.notes, i)
            end
        end
    end
end

command.NoteAdd = NoteAdd

command.NewChannel, command.RemoveChannel, command.NewEffect, command.RemoveEffect, command.ReorderEffect =
    unpack(require("command_channel"))

return command
