local backend = require("backend")
local build = require("build")
local SliderValue = require("ui/slider_value")
local deviceList = require("device_list")

local function newDeviceData(name, options)
    local state = {}

    for i, v in ipairs(options.parameters) do
        local widget_type = v[2]
        local widget_options = v[3] or {}

        if widget_type == "slider" then
            local sv = SliderValue:new(widget_options)
            state[i] = sv.default
        elseif widget_type == "selector" then
            state[i] = widget_options.default or 1
        elseif widget_type == "toggle" then
            state[i] = widget_options.default or false
        else
            error(widget_type .. " not supported!")
        end

        assert(state[i] ~= nil)
    end

    return { name = name, state = state }
end

local function _removeChannel(ch_index)
    if selection.channel_index == ch_index then
        selection.channel_index = nil
    end
    table.remove(project.channels, ch_index)
    table.remove(ui_channels, ch_index)
    backend:removeChannel(ch_index)
end

local function _removeEffect(ch_index, effect_index)
    if selection.channel_index == ch_index and selection.device_index == effect_index then
        selection.device_index = nil
    end

    table.remove(project.channels[ch_index].effects, effect_index)
    table.remove(ui_channels[ch_index].effects, effect_index)
    backend:removeEffect(ch_index, effect_index)
end

local function _reorderEffect(ch_index, old_index, new_index)
    if project.channels[ch_index] then
        local n = #project.channels[ch_index].effects

        if old_index >= 1 and old_index <= n and new_index >= 1 and new_index <= n then
            local ch = project.channels[ch_index]
            local temp = table.remove(ch.effects, old_index)
            table.insert(ch.effects, new_index, temp)

            ch = ui_channels[ch_index]
            temp = table.remove(ch.effects, old_index)
            table.insert(ch.effects, new_index, temp)

            backend:reorderEffect(ch_index, old_index, new_index)

            if selection.channel_index == ch_index and selection.device_index == old_index then
                selection.device_index = new_index
            end
        end
    end
end

--
local newChannel = {}
newChannel.__index = newChannel

function newChannel.new(name)
    local self = setmetatable({}, newChannel)

    self.name = name
    self.ch_index = #project.channels + 1

    assert(self.ch_index)
    return self
end

function newChannel:run()
    local options = deviceList.instruments[self.name]
    assert(options)
    -- build state
    local channel = {
        instrument = newDeviceData(self.name, options),
        effects = {},
        mute = false,
        solo = false,
        armed = false,
        visible = true,
        lock = false,
        name = self.name .. " " .. #project.channels,
    }
    table.insert(project.channels, channel)

    build.channel(channel)

    -- select it
    selection.channel_index = self.ch_index

    return channel
end

function newChannel:reverse()
    _removeChannel(self.ch_index)
end

--
local removeChannel = {}
removeChannel.__index = removeChannel

function removeChannel.new(ch_index)
    local self = setmetatable({}, removeChannel)

    self.ch_index = ch_index
    self.channel = util.clone(project.channels[ch_index])
    return self
end

function removeChannel:run()
    _removeChannel(self.ch_index)
end

function removeChannel:reverse()
    local channel = util.clone(self.channel)
    table.insert(project.channels, self.ch_index, channel)
    build.channel(channel)
end

--
local newEffect = {}
newEffect.__index = newEffect

function newEffect.new(ch_index, name)
    local self = setmetatable({}, newEffect)

    self.ch_index = ch_index
    self.effect_index = #project.channels[ch_index].effects + 1
    self.name = name
    return self
end

function newEffect:run()
    local options = deviceList.effects[self.name]
    assert(options)

    local effect = newDeviceData(self.name, options)
    table.insert(project.channels[self.ch_index].effects, effect)

    build.effect(self.ch_index, self.effect_index, effect)

    -- select it
    selection.device_index = self.effect_index
end

function newEffect:reverse()
    _removeEffect(self.ch_index, self.effect_index)
end

--
local removeEffect = {}
removeEffect.__index = removeEffect

function removeEffect.new(ch_index, effect_index)
    local self = setmetatable({}, removeEffect)

    self.ch_index = ch_index
    self.effect_index = effect_index
    self.effect = util.clone(project.channels[ch_index].effects[effect_index])
    return self
end

function removeEffect:run()
    _removeEffect(self.ch_index, self.effect_index)
end

function removeEffect:reverse()
    local effect = util.clone(self.effect)
    table.insert(project.channels[self.ch_index].effects, self.effect_index, effect)
    build.effect(self.ch_index, self.effect_index, effect)
end

--
local reorderEffect = {}
reorderEffect.__index = reorderEffect

function reorderEffect.new(ch_index, old_index, new_index)
    local self = setmetatable({}, reorderEffect)

    self.ch_index = ch_index
    self.old_index = old_index
    self.new_index = new_index
    return self
end

function reorderEffect:run()
    _reorderEffect(self.ch_index, self.old_index, self.new_index)
end

function reorderEffect:reverse()
    _reorderEffect(self.ch_index, self.new_index, self.old_index)
end

return { newChannel, removeChannel, newEffect, removeEffect, reorderEffect }
