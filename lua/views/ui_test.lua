local View = require("view")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")

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
	new.dropdown = widgets.Dropdown:new({ list = { "AAA", "BBB", "CCC" }, has_state = true })
	new.slider = widgets.Slider:new({ min = 20, max = 20000, fmt = "Hz", t = "log" })
	new.checkbox = widgets.Toggle:new("checkbox widget", {})
	new.selector = widgets.Selector:new({ "one", "two", "three" })
	new.toggle = widgets.Toggle:new("toggle widget", { style = "toggle" })

	return new
end

function UiTest:update()
	self.ui:startFrame()

	self.ui:label("left aligned label")

	self.ui.layout:col(self.w * 0.5)
	self.ui.layout:col(self.w * 0.3)
	self.dropdown:update(self.ui, self.state, "combo")
	self.ui.layout:newRow()

	self.ui.layout:col(self.w * 0.5)
	self.ui:label("center aligned", "center")

	self.ui.layout:col(self.w * 0.3)
	if self.button:update(self.ui) then
		command.run_and_register(command.change.new(self.state, "toggle", not self.state.toggle))
	end
	self.ui.layout:newRow()

	local w_label = self.w * 0.3
	self.ui.layout:col(w_label)

	self.ui:label("a slider", "right")
	self.ui.layout:col(self.w - w_label)
	self.slider:update(self.ui, self.state, "slider")

	self.checkbox:update(self.ui, self.state, "toggle")

	self.ui.layout:col(self.w * 0.5)
	self.toggle:update(self.ui, self.state, "toggle")

	self.ui.layout:col(self.w * 0.5)
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
