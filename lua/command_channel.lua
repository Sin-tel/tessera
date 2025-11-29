local SliderValue = require("ui/slider_value")
local build = require("build")
local deviceList = require("device_list")

local function minHueDist(hue)
    -- calculate distance to closest hue that already exists
    local min_dist = 180.0
    for i, v in ipairs(project.channels) do
        -- distance in degrees
        local a = math.abs(hue - v.hue - 360.0 * math.floor(0.5 + (hue - v.hue) / 360.0))
        if a < min_dist then
            min_dist = a
        end
    end
    return min_dist
end

local function findHue()
    -- try some random hues, pick  the one that is furthest away from existing ones
    local hue = math.random() * 360.0
    local min_dist = minHueDist(hue)
    for i = 1, 10 do
        local p_hue = math.random() * 360.0
        local p_min_dist = minHueDist(p_hue)
        if p_min_dist > min_dist then
            hue = p_hue
            min_dist = p_min_dist
        end
    end
    return hue
end

local function newDeviceData(name, options)
    local state = {}

    for i, v in ipairs(options.parameters) do
        local widget_type = v[2]
        local widget_options = v[3] or {}

        if widget_type == "slider" then
            local sv = SliderValue.new(widget_options)
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

local function newChannelData(name, options)
    return {
        instrument = newDeviceData(name, options),
        effects = {},
        notes = {},
        control = {},
        mute = false,
        solo = false,
        armed = false,
        visible = true,
        lock = false,
        hue = findHue(),
        name = name .. " " .. #project.channels,
    }
end

local function _removeChannel(ch_index)
    if selection.ch_index == ch_index then
        selection.ch_index = nil
    end
    table.remove(project.channels, ch_index)
    table.remove(ui_channels, ch_index)
    build.refresh_channels()
    tessera.audio.removeChannel(ch_index)
end

local function _removeEffect(ch_index, effect_index)
    if selection.ch_index == ch_index and selection.device_index == effect_index then
        selection.device_index = nil
    end

    table.remove(project.channels[ch_index].effects, effect_index)
    table.remove(ui_channels[ch_index].effects, effect_index)
    tessera.audio.removeEffect(ch_index, effect_index)
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

            tessera.audio.reorderEffect(ch_index, old_index, new_index)

            if selection.ch_index == ch_index and selection.device_index == old_index then
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
    local channel = newChannelData(self.name, options)
    table.insert(project.channels, channel)

    build.channel(self.ch_index, channel)

    -- select it
    selection.ch_index = self.ch_index

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
    build.channel(self.ch_index, channel)
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
