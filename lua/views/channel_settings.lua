local Menu = require("menu")
local Ui = require("ui/ui")
local View = require("view")
local device_list = require("device_list")
local widgets = require("ui/widgets")

local ChannelSettings = View.derive("Channel settings")
ChannelSettings.__index = ChannelSettings

function ChannelSettings.new()
	local self = setmetatable({}, ChannelSettings)

	self.ui = Ui.new(self)

	-- make list of effect (name, key) and sort them
	self.effect_list = {}
	for key, v in pairs(device_list.effects) do
		if not v.hide or not release then
			table.insert(self.effect_list, { v.name, key })
		end
	end
	table.sort(self.effect_list, function(a, b)
		return a[1] < b[1]
	end)

	self.dropdown = widgets.Button.new("Add effect")

	return self
end

function ChannelSettings:update()
	self.ui:start_frame()
	self.ui.layout:col(Ui.scale(120))

	if self.dropdown:update(self.ui) then
		workspace:set_overlay(self:menu())
	end

	if selection.ch_index then
		local ch = ui_channels[selection.ch_index]

		if ch.instrument then
			if ch.instrument:update(self.ui, 0, self.w) then
				selection.device_index = 0
			end
		end

		for i, v in ipairs(ch.effects) do
			if v:update(self.ui, i, self.w) then
				selection.device_index = i
			end
		end

		if self.add_effect_index then
			local key = self.effect_list[self.add_effect_index][2]
			command.run_and_register(command.NewEffect.new(selection.ch_index, key))
			self.add_effect_index = nil
		end
	end
	self.ui:end_frame()
end

function ChannelSettings:draw()
	self.ui:draw()
end

function ChannelSettings:menu()
	local options = {
		style = "menu",
		align = tessera.graphics.ALIGN_LEFT,
	}
	local items = {}
	for i, v in ipairs(self.effect_list) do
		table.insert(items, {
			widget = widgets.Button.new(v[1], options),
			action = function()
				self.add_effect_index = i
			end,
		})
	end

	return Menu.new(items)
end

return ChannelSettings
