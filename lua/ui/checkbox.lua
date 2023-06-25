local Ui = require("ui/ui")
local util = require("util")

local Checkbox = {}

local CORNER_RADIUS = 4

function Checkbox:new(text)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.text = text
	new.checked = false

	return new
end

function Checkbox:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)

	if ui.clicked == self then
		self.checked = not self.checked
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line

	if self.checked then
		color_fill = theme.widget
	end
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	ui:pushDraw(self.draw, self, color_fill, color_line, x, y, w, h)

	return self.checked
end

-- TODO: dirty flag
function Checkbox:getFloat()
	if self.checked then
		return 1.0
	else
		return 0.0
	end
end

function Checkbox:draw(color_fill, color_line, x, y, w, h)
	love.graphics.setColor(color_fill)
	love.graphics.rectangle("fill", x, y, h, h, CORNER_RADIUS)
	love.graphics.setColor(color_line)
	love.graphics.rectangle("line", x, y, h, h, CORNER_RADIUS)
	love.graphics.setColor(theme.ui_text)
	local left_pad = h + Ui.DEFAULT_PAD
	util.drawText(self.text, x + left_pad, y, w - left_pad, h)
end

return Checkbox
