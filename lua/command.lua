local command = {}

command.maxSize = 50
command.stack = {}
command.index = 0

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

return command
