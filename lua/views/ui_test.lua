local View = require("view")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")
local command = require("command")

local UiTest = View:derive("UI test")

function UiTest:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	self.state = {
		toggle = false,
		combo = 1,
		slider = 200,
	}

	new.ui = Ui:new(new)

	new.button = widgets.Button:new("click me")
	new.dropdown = widgets.Dropdown:new({ list = { "AAA", "BBB", "CCC" } })
	new.slider = widgets.Slider:new({ min = 20, max = 20000, fmt = "Hz", t = "log" })
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
	self.dropdown:update(self.ui, self.state, "combo")
	self.ui.layout:newRow()

	self.ui.layout:col(w * 0.5)
	self.ui:label("center aligned", "center")

	self.ui.layout:col(w * 0.3)
	if self.button:update(self.ui) then
		local c = command.change.new(self.state, "toggle", not self.state.toggle)
		c:run()
		command.register(c)
	end
	self.ui.layout:newRow()

	local w_label = w * 0.3
	self.ui.layout:col(w_label)

	self.ui:label("a slider", "right")
	self.ui.layout:col(w - w_label)
	self.slider:update(self.ui, self.state, "slider")

	self.checkbox:update(self.ui, self.state, "toggle")

	self.ui.layout:col(w * 0.5)
	self.toggle:update(self.ui, self.state, "toggle")

	self.ui.layout:col(w * 0.5)
	self.selector:update(self.ui, self.state, "combo")

	-- if self.state.toggle then
	-- 	self.ui:label("pew!", "center")
	-- end

	self.ui:endFrame()
end

function UiTest:draw()
	self.ui:draw()
end

return UiTest
