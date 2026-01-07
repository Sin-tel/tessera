local build = require("build")
local device_list = require("device_list")

local function remove_channel(ch_index)
    table.remove(project.channels, ch_index)
    table.remove(ui_channels, ch_index)
    build.refresh_channels()
    tessera.audio.remove_channel(ch_index)
    if selection.ch_index == ch_index then
        selection.select_default_channel()
    end
end

local function remove_effect(ch_index, effect_index)
    if selection.ch_index == ch_index and selection.device_index == effect_index then
        selection.device_index = nil
    end

    table.remove(project.channels[ch_index].effects, effect_index)
    table.remove(ui_channels[ch_index].effects, effect_index)
    tessera.audio.remove_effect(ch_index, effect_index)
end

local function reorder_effect(ch_index, old_index, new_index)
    if project.channels[ch_index] then
        local n = #project.channels[ch_index].effects

        if old_index >= 1 and old_index <= n and new_index >= 1 and new_index <= n then
            local ch = project.channels[ch_index]
            local temp = table.remove(ch.effects, old_index)
            table.insert(ch.effects, new_index, temp)

            ch = ui_channels[ch_index]
            temp = table.remove(ch.effects, old_index)
            table.insert(ch.effects, new_index, temp)

            tessera.audio.reorder_effect(ch_index, old_index, new_index)

            if selection.ch_index == ch_index and selection.device_index == old_index then
                selection.device_index = new_index
            end
        end
    end
end

--
local NewChannel = {}
NewChannel.__index = NewChannel

function NewChannel.new(name)
    local self = setmetatable({}, NewChannel)

    self.name = name
    self.ch_index = #project.channels + 1

    assert(self.ch_index)
    return self
end

function NewChannel:run()
    local options = device_list.instruments[self.name]
    assert(options)
    -- build state
    options.instrument_key = self.name
    local channel = build.new_channel_data(options)
    table.insert(project.channels, channel)

    build.channel(self.ch_index, channel)

    -- select it
    selection.ch_index = self.ch_index
    selection.device_index = nil

    return channel
end

function NewChannel:reverse()
    remove_channel(self.ch_index)
end

--
local RemoveChannel = {}
RemoveChannel.__index = RemoveChannel

function RemoveChannel.new(ch_index)
    local self = setmetatable({}, RemoveChannel)

    self.ch_index = ch_index
    self.channel = util.clone(project.channels[ch_index])
    return self
end

function RemoveChannel:run()
    remove_channel(self.ch_index)
end

function RemoveChannel:reverse()
    local channel = util.clone(self.channel)
    table.insert(project.channels, self.ch_index, channel)
    build.channel(self.ch_index, channel)
end

--
local NewEffect = {}
NewEffect.__index = NewEffect

function NewEffect.new(ch_index, name)
    local self = setmetatable({}, NewEffect)

    self.ch_index = ch_index
    self.effect_index = #project.channels[ch_index].effects + 1
    self.name = name
    self.display_name = name
    return self
end

function NewEffect:run()
    local options = device_list.effects[self.name]
    assert(options)

    local effect = build.new_device_data(self.name, options)
    table.insert(project.channels[self.ch_index].effects, effect)

    build.effect(self.ch_index, self.effect_index, effect)

    -- select it
    selection.device_index = self.effect_index
end

function NewEffect:reverse()
    remove_effect(self.ch_index, self.effect_index)
end

--
local RemoveEffect = {}
RemoveEffect.__index = RemoveEffect

function RemoveEffect.new(ch_index, effect_index)
    local self = setmetatable({}, RemoveEffect)

    self.ch_index = ch_index
    self.effect_index = effect_index
    self.effect = util.clone(project.channels[ch_index].effects[effect_index])
    return self
end

function RemoveEffect:run()
    remove_effect(self.ch_index, self.effect_index)
end

function RemoveEffect:reverse()
    local effect = util.clone(self.effect)
    table.insert(project.channels[self.ch_index].effects, self.effect_index, effect)
    build.effect(self.ch_index, self.effect_index, effect)
end

--
local ReorderEffect = {}
ReorderEffect.__index = ReorderEffect

function ReorderEffect.new(ch_index, old_index, new_index)
    local self = setmetatable({}, ReorderEffect)

    self.ch_index = ch_index
    self.old_index = old_index
    self.new_index = new_index
    return self
end

function ReorderEffect:run()
    reorder_effect(self.ch_index, self.old_index, self.new_index)
end

function ReorderEffect:reverse()
    reorder_effect(self.ch_index, self.new_index, self.old_index)
end

return { NewChannel, RemoveChannel, NewEffect, RemoveEffect, ReorderEffect }
