local View = require("view")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")
local util = require("util")

local UiTest = View:derive("UI test")

function UiTest:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.ui = Ui:new(new)

	new.button = widgets.Button:new("click me")
	new.slider = widgets.Slider:new({ default = 2000, min = 20, max = 20000, fmt = "Hz", t = "log" })
	new.checkbox = widgets.Checkbox:new("checkbox")

	return new
end

function UiTest:update()
	local w, h = self:getDimensions()

	self.ui:startFrame()
	self.ui.layout:start()

	self.ui:label("centered label", "center")

	self.ui.layout:col(w * 0.5)
	self.ui:label("right aligned", "right")
	self.ui.layout:col(w * 0.3)
	self.ui.layout:newRow()

	if self.ui:put(self.button) then
		self.show = not self.show
	end

	if self.show then
		self.ui:label("pew!", "center")
	end

	local w_label = w * 0.3
	self.ui.layout:col(w_label)

	self.ui:label("a slider")
	self.ui.layout:col(w - w_label)
	self.ui:put(self.slider)

	if self.ui:put(self.checkbox) then
		--
	end
end

function UiTest:draw()
	self.ui:draw()
end

return UiTest
