-- small gui framework
-- loosely inspired by egui (https://github.com/emilk/egui) and SUIT (https://github.com/vrld/suit/)
local mouse = require("mouse")
local Ui = {}

-- TODO: set background per row
-- TODO: separators
-- TODO: option picker
-- TODO: editable labels
-- TODO: radio buttons
-- TODO: combobox

Ui.RESIZE_W = 5
Ui.MIN_SIZE = 32
Ui.HEADER = 32
Ui.BORDER_RADIUS = 4
Ui.BORDER_SIZE = 1
Ui.RIBBON_HEIGHT = 32

Ui.ROW_HEIGHT = 28
Ui.PARAMETER_LABEL_WIDTH = 150 -- max width of parameter labels
Ui.PARAMETER_PAD = 8 -- padding for parameters
Ui.BUTTON_SMALL = 18

Ui.DEFAULT_PAD = 5

function Ui:new(view)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.draw_queue = {}
	new.view = view

	new.hover = false
	new.active = false
	new.clicked = false
	new.was_active = false

	local Layout = require("ui/layout")
	new.layout = Layout:new()

	return new
end

function Ui:startFrame()
	self.mx, self.my = self.view:getMouse()

	-- TODO: should these be global?
	-- at least make them static
	self.hover = false
	self.clicked = false
	if mouse.button_released then
		self.was_active = self.active
		self.active = false
	end
end

function Ui:next()
	if not self.layout.ok then
		-- TODO: do col("max") when in column mode
		local w, h = self.view:getDimensions()
		self.layout:row(w, Ui.ROW_HEIGHT)
	end
end

function Ui:put(widget)
	self:next()
	local ret = widget:update(self, self.layout:get())
	return ret
end

local function drawLabel(text, align, x, y, w, h)
	love.graphics.setColor(theme.ui_text)
	util.drawText(text, x, y, w, h, align)
end

function Ui:label(text, align)
	self:next()
	self:pushDraw(drawLabel, text, align, self.layout:get())
end

function Ui:hitbox(widget, x, y, w, h)
	if self.view:focus() and self.mx >= x and self.my >= y and self.mx <= x + w and self.my <= y + h then
		if mouse.button_pressed == 1 then
			self.active = widget
		end
		if mouse.button_released == 1 and self.was_active == widget then
			self.clicked = widget
		end
		if not self.active or self.active == widget then
			self.hover = widget
		end

		return true
	end
	return false
end

function Ui:pushDraw(f, ...)
	local args = { ... }
	table.insert(self.draw_queue, function()
		f(unpack(args))
	end)
end

function Ui:draw()
	for _, f in ipairs(self.draw_queue) do
		f()
	end
	-- TODO: maybe we can cache these?
	self.draw_queue = {}
end

return Ui