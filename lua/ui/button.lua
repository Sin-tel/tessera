local Ui = require("ui/ui")
local util = require("util")

local Button = {}

local CORNER_RADIUS = 4

function Button:new(text)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.text = text

	return new
end

function Button:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)

	local color_fill = theme.widget_bg
	local color_line = theme.line
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	ui:pushDraw(self.draw, self, color_fill, color_line, x, y, w, h)

	return ui.clicked == self
end

function Button:draw(color_fill, color_line, x, y, w, h)
	if w > 10 then
		love.graphics.setColor(color_fill)
		love.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)
		love.graphics.setColor(color_line)
		love.graphics.rectangle("line", x, y, w, h, CORNER_RADIUS)
		love.graphics.setColor(theme.ui_text)
		util.drawText(self.text, x, y, w, h, "center")
	end
end

return Button
