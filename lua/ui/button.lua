local Ui = require("ui/ui")

local Button = {}
Button.__index = Button

function Button.new(text)
	local self = setmetatable({}, Button)

	self.text = text

	return self
end

function Button:update(ui)
	local x, y, w, h = ui:next()
	ui:hitbox(self, x, y, w, h)

	ui:pushDraw(self.draw, { self, ui, x, y, w, h })

	return ui.clicked == self
end

function Button:draw(ui, x, y, w, h)
	if w > 10 then
		local color_fill = theme.widget_bg
		local color_line = theme.line
		if ui.active == self then
			color_fill = theme.widget_press
		end
		if ui.hover == self and ui.active ~= self then
			color_line = theme.line_hover
		end
		love.graphics.setColor(color_fill)
		love.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
		love.graphics.setColor(color_line)
		love.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
		love.graphics.setColor(theme.ui_text)
		util.drawText(self.text, x, y, w, h, "center")
	end
end

return Button
