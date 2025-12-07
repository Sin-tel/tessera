local Ui = require("ui/ui")
local View = require("view")
local widgets = require("ui/widgets")

local UiTest = View.derive("UI test")
UiTest.__index = UiTest

function UiTest.new()
	local self = setmetatable({}, UiTest)

	self.state = {
		toggle = 0,
		combo = 1,
		slider = 200,
	}

	self.ui = Ui.new(self)

	self.button = widgets.Button.new("click me")
	self.dropdown = widgets.Dropdown.new({ list = { "AAA", "BBB", "CCC" }, has_state = true })
	self.slider = widgets.Slider.new({ min = 20, max = 20000, fmt = "Hz", t = "log" })
	self.checkbox = widgets.Toggle.new("checkbox widget", {})
	self.selector = widgets.Selector.new({ "one", "two", "three" })
	self.toggle = widgets.Toggle.new("toggle widget", { style = "toggle" })

	return self
end

function UiTest:update()
	self.ui:start_frame()

	self.ui.layout:col(self.w * 0.5)
	self.ui:label("left aligned label")
	self.ui.layout:col(self.w * 0.3)
	self.dropdown:update(self.ui, self.state, "combo")
	self.ui.layout:new_row()

	self.ui.layout:col(self.w * 0.5)
	self.ui:label("center aligned", tessera.graphics.ALIGN_CENTER)

	self.ui.layout:col(self.w * 0.3)
	if self.button:update(self.ui) then
		local new_text = self.button.text .. "!"
		if self.button.text == "click me" then
			new_text = "click!"
		end
		command.run_and_register(command.Change.new(self.button, "text", new_text))
	end
	self.ui.layout:new_row()

	local w_label = self.w * 0.3
	self.ui.layout:col(w_label)

	self.ui:label("a slider", tessera.graphics.ALIGN_RIGHT)
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

	self.ui:end_frame()
end

function UiTest:draw()
	self.ui:draw()
end

return UiTest
