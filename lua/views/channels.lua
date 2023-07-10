local Ui = require("ui/ui")
local deviceList = require("device_list")
local widgets = require("ui/widgets")
local View = require("view")

local Channels = View:derive("Channels")

function Channels:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	-- new.select = nil
	new.ui = Ui:new(new)

	new.intrument_list = {}
	for k, v in pairs(deviceList.instruments) do
		table.insert(new.intrument_list, k)
	end
	new.dropdown = widgets.Dropdown:new({ title = "add instrument", list = new.intrument_list })

	return new
end

function Channels:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.ui:startFrame()
	self.ui.layout:padding()
	self.ui.layout:row(w)
	local add_instrument_index = self.ui:put(self.dropdown)

	self.ui.layout:padding(0)

	if add_instrument_index then
		local intrument_name = self.intrument_list[add_instrument_index]
		local ch = channelHandler:add(intrument_name)
	end

	for _, v in ipairs(channelHandler.list) do
		self.ui:put(v.widget)
	end

	self.ui:endFrame()
end

function Channels:draw()
	self.ui:draw()
end

return Channels
