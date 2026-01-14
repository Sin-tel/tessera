local Ui = require("ui/ui")
local View = require("view")
local widgets = require("ui/widgets")

local UiTest = View.derive("UI test")
UiTest.__index = UiTest

function UiTest.new()
	local self = setmetatable({}, UiTest)

	self.state = {
		toggle = false,
		combo = 1,
		slider = 200,
	}

	self.ui = Ui.new(self)

	self.button = widgets.Button.new("click me")
	self.dropdown = widgets.Dropdown.new(self.state, "combo", { list = { "AAA", "BBB", "CCC" } })
	self.collapse = widgets.Collapse.new("click to reveal")
	self.slider = widgets.Slider.new(self.state, "slider", { min = 20, max = 20000, fmt = "Hz", t = "log" })
	self.checkbox = widgets.Toggle.new(self.state, "toggle", { label = "checkbox widget" })
	self.selector = widgets.Selector.new(self.state, "combo", { list = { "one", "two", "three" } })
	self.toggle = widgets.Toggle.new(self.state, "toggle", { label = "toggle widget", style = "toggle" })

	return self
end

function UiTest:update()
	self.ui:start_frame()

	self.ui.layout:col(self.w * 0.5)
	self.ui:label("left aligned label")
	self.ui.layout:col(self.w * 0.3)
	self.dropdown:update(self.ui)
	self.ui.layout:new_row()

	self.ui.layout:col(self.w * 0.5)
	self.ui:label("center aligned", { align = tessera.graphics.ALIGN_CENTER })

	self.ui.layout:col(self.w * 0.3)
	if self.button:update(self.ui) then
		local new_text = self.button.text .. "!"
		if self.button.text == "click me" then
			new_text = "click!"
		end
		command.run_and_register(command.Change.new(self.button, "text", new_text))
	end
	self.ui.layout:new_row()

	if self.collapse:update(self.ui) then
		self.ui:background(theme.bg_nested)

		local w_label = self.w * 0.3
		self.ui.layout:col(w_label)
		self.ui:label("a slider", { align = tessera.graphics.ALIGN_RIGHT })
		self.ui.layout:col(self.w - w_label)
		self.slider:update(self.ui)
		self.ui:background()
	end

	self.checkbox:update(self.ui)

	self.ui.layout:col(self.w * 0.5)
	self.toggle:update(self.ui)

	self.ui.layout:col(self.w * 0.5)
	self.selector:update(self.ui)

	-- if self.state.toggle then
	-- 	self.ui:label("pew!", "center")
	-- end

	self.ui:end_frame()
end

function UiTest:draw()
	self.ui:draw()
end

return UiTest
