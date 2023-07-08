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
	-- new.dropdown = widgets.Dropdown:new({ list = { "option1", "option2", "option3", "option4", "option5" } })
	new.dropdown =
		widgets.Dropdown:new({ title = "hello", list = { "option1", "option2", "option3", "option4", "option5" } })
	new.slider = widgets.Slider:new({ default = 700, min = 20, max = 20000, fmt = "Hz", t = "log" })
	new.checkbox = widgets.Toggle:new("checkbox widget", {})
	new.selector = widgets.Selector:new({ "one", "two", "three" })
	new.toggle = widgets.Toggle:new("toggle widget", { style = "toggle" })

	return new
end

function UiTest:update()
	local w, h = self:getDimensions()

	self.ui:startFrame()

	self.ui:label("left aligned label")

	self.ui.layout:col(w * 0.5)
	self.ui.layout:col(w * 0.3)
	self.ui:put(self.dropdown)
	self.ui.layout:newRow()

	self.ui.layout:col(w * 0.5)
	self.ui:label("center aligned", "center")

	self.ui.layout:col(w * 0.3)
	if self.ui:put(self.button) then
		self.show = not self.show
	end
	self.ui.layout:newRow()

	if self.show then
		self.ui:label("pew!", "center")
	end

	local w_label = w * 0.3
	self.ui.layout:col(w_label)

	self.ui:label("a slider", "right")
	self.ui.layout:col(w - w_label)
	self.ui:put(self.slider)

	if self.ui:put(self.checkbox) then
		--
	end

	self.ui.layout:col(w * 0.5)
	self.ui:put(self.toggle)
	self.ui.layout:col(w * 0.5)
	self.ui:put(self.selector)

	self.ui:endFrame()
end

function UiTest:draw()
	self.ui:draw()
end

return UiTest
