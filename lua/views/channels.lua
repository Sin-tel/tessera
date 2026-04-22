local Menu = require("menu")
local Ui = require("ui/ui")
local View = require("view")
local device_list = require("device_list")
local save = require("save")
local widgets = require("ui/widgets")

local Channels = View.derive("Channels")
Channels.__index = Channels

function Channels.new()
	local self = setmetatable({}, Channels)

	self.ui = Ui.new(self)

	-- make list of instruments and sort them
	self.intrument_list = {}
	for key, v in pairs(device_list.instruments) do
		if (not v.hide or not release) and key ~= "vst_instrument" then
			local instrument = util.clone(v)
			local options = {
				name = instrument.display_name,
				instrument = instrument,
			}
			table.insert(self.intrument_list, options)
		end
	end

	-- add plugins dynamically
	local plugins = save.read_plugins()
	for _, v in ipairs(plugins) do
		if v.is_instrument then
			local instrument = util.clone(device_list.instruments.vst_instrument)
			instrument.display_name = "VST: " .. v.name
			instrument.plugin = {
				descriptor = v,
			}

			local options = {
				name = instrument.display_name,
				instrument = instrument,
			}
			table.insert(self.intrument_list, options)
		end
	end

	table.sort(self.intrument_list, function(a, b)
		return a.name < b.name
	end)

	self.dropdown = widgets.Button.new("Add channel")

	return self
end

function Channels:update()
	self.ui:start_frame()
	self.ui.layout:padding()
	self.ui.layout:col(Ui.scale(120))
	if self.dropdown:update(self.ui) then
		workspace:set_overlay(self:menu())
	end

	if self.add_instrument_index then
		local options = self.intrument_list[self.add_instrument_index]
		command.run_and_register(command.NewChannel.new(options))
		self.add_instrument_index = nil
	end
	self.ui.layout:new_row()

	for i, ch in ipairs(ui_channels) do
		-- background has a frame delay since we have to layout first
		local bg_color = theme.background
		if self.hover == ch then
			bg_color = theme.bg_highlight
		end
		if selection.ch_index == i then
			bg_color = theme.bg_focus
		end
		if ch:update(self.ui, i, bg_color, self.w) then
			selection.ch_index = i
			selection.device_index = nil
		end
	end
	-- remember for next frame
	self.hover = self.ui.hover

	self.ui:end_frame()
end

function Channels:draw()
	self.ui:draw()
end

function Channels:menu()
	local options = {
		style = "menu",
		align = tessera.graphics.ALIGN_LEFT,
	}
	local items = {}
	for i, v in ipairs(self.intrument_list) do
		table.insert(items, {
			widget = widgets.Button.new(v.name, options),
			action = function()
				self.add_instrument_index = i
			end,
		})
	end

	return Menu.new(items)
end

return Channels
