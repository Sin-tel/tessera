local Ui = require("ui/ui")
local View = require("view")
local device_list = require("device_list")
local widgets = require("ui/widgets")

local Channels = View.derive("Channels")
Channels.__index = Channels

function Channels.new()
	local self = setmetatable({}, Channels)

	self.ui = Ui.new(self)

	self.intrument_list = {}
	for key in pairs(device_list.instruments) do
		table.insert(self.intrument_list, key)
	end
	table.sort(self.intrument_list)

	self.dropdown = widgets.Dropdown.new(nil, nil, { title = "add channel", list = self.intrument_list })

	return self
end

function Channels:update()
	self.ui:start_frame()
	self.ui.layout:padding()
	self.ui.layout:col(self.w * 0.33)
	local add_instrument_index = self.dropdown:update(self.ui)

	self.ui.layout:padding(0)
	if add_instrument_index then
		local intrument_name = self.intrument_list[add_instrument_index]

		command.run_and_register(command.NewChannel.new(intrument_name))
	end

	for i, v in ipairs(ui_channels) do
		v.widget:update(self.ui, i)
	end

	self.ui:end_frame()
end

function Channels:draw()
	self.ui:draw()
end

return Channels
