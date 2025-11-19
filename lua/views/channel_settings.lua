local Ui = require("ui/ui")
local View = require("view")
local deviceList = require("device_list")
local widgets = require("ui/widgets")

local ChannelSettings = View.derive("Channel settings")
ChannelSettings.__index = ChannelSettings

function ChannelSettings.new()
	local self = setmetatable({}, ChannelSettings)

	self.ui = Ui.new(self)

	self.effect_list = {}
	for k, v in pairs(deviceList.effects) do
		table.insert(self.effect_list, k)
	end
	table.sort(self.effect_list)
	self.dropdown = widgets.Dropdown.new({ title = "add effect", list = self.effect_list })

	return self
end

function ChannelSettings:update()
	self.ui:startFrame()
	self.ui.layout:row(self.w)
	local add_effect_index = self.dropdown:update(self.ui)

	-- TODO: should calculate this in device instead
	local w_label = util.clamp(self.w * 0.4 - 64, 0, Ui.PARAMETER_LABEL_WIDTH)

	if selection.ch_index then
		local ch = ui_channels[selection.ch_index]
		if ch.instrument:update(self.ui, 0, self.w, w_label) then
			selection.device_index = 0
		end

		for i, v in ipairs(ch.effects) do
			if v:update(self.ui, i, self.w, w_label) then
				selection.device_index = i
			end
		end

		if add_effect_index then
			local effect_name = self.effect_list[add_effect_index]
			command.run_and_register(command.newEffect.new(selection.ch_index, effect_name))
		end
	end
	self.ui:endFrame()
end

function ChannelSettings:draw()
	self.ui:draw()
end

return ChannelSettings
