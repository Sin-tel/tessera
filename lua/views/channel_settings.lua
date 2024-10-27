local View = require("view")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")
local util = require("util")
local deviceList = require("device_list")
local channelHandler = require("channel_handler")

local ChannelSettings = View:derive("Channel settings")

function ChannelSettings:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.ui = Ui:new(new)

	new.effect_list = {}
	for k, v in pairs(deviceList.effects) do
		table.insert(new.effect_list, k)
	end
	new.dropdown = widgets.Dropdown:new({ title = "add effect", list = new.effect_list })

	return new
end

function ChannelSettings:update()
	local w, h = self:getDimensions()

	self.ui:startFrame()
	self.ui.layout:row(w)
	local add_effect_index = self.dropdown:update(self.ui)

	-- TODO: should calculate this in device instead
	local w_label = util.clamp(w * 0.4 - 64, 0, Ui.PARAMETER_LABEL_WIDTH)

	if selection.channel then
		if selection.channel.instrument:updateUi(self.ui, w, w_label) then
			selection.device = selection.channel.instrument
		end
		for i, v in ipairs(selection.channel.effects) do
			if v:updateUi(self.ui, w, w_label) then
				selection.device = v
			end
		end

		-- TODO: command
		if add_effect_index then
			local effect_name = self.effect_list[add_effect_index]
			local new_effect = channelHandler:addEffect(selection.channel, effect_name)
			-- this has to be set here otherwise clicking on the dropdown will select the device underneath
			selection.device = new_effect
		end
	end
	self.ui:endFrame()
end

function ChannelSettings:draw()
	self.ui:draw()
end

return ChannelSettings
