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

	ui:push_draw(self.draw, { self, ui, x, y, w, h })

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
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
		tessera.graphics.set_color(color_line)
		tessera.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
		tessera.graphics.set_color(theme.ui_text)
		util.draw_text(self.text, x, y, w, h, "center")
	end
end

return Button
