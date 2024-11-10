local View = require("view")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")
local deviceList = require("device_list")

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

	if selection.channel_index then
		local ch = ui_channels[selection.channel_index]
		if ch.instrument:update(self.ui, 0, w, w_label) then
			selection.device_index = 0
		end

		for i, v in ipairs(ch.effects) do
			if v:update(self.ui, i, w, w_label) then
				selection.device_index = i
			end
		end

		if add_effect_index then
			local effect_name = self.effect_list[add_effect_index]
			command.run_and_register(command.newEffect.new(selection.channel_index, effect_name))
		end
	end
	self.ui:endFrame()
end

function ChannelSettings:draw()
	self.ui:draw()
end

return ChannelSettings
