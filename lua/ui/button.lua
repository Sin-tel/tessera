local Ui = require("ui/ui")

local Button = {}
Button.__index = Button

function Button.new(text, options)
	local self = setmetatable({}, Button)

	self.text = text

	options = options or {}
	self.align = options.align or tessera.graphics.ALIGN_CENTER
	self.style = options.style or "normal"
	self.indent = options.indent or 0
	self.text_color = options.text_color or theme.ui_text

	return self
end

function Button:update(ui)
	local x, y, w, h = ui:next()
	ui:hitbox(self, x, y, w, h)

	if self.style == "normal" then
		ui:push_draw(self.draw, { self, ui, x, y, w, h })
	elseif self.style == "menu" then
		ui:push_draw(self.draw_menu, { self, ui, x, y, w, h })
	else
		error("unreachable")
	end

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
		tessera.graphics.set_color(self.text_color)
		tessera.graphics.label(self.text, x, y, w, h, self.align)
	end
end

function Button:draw_menu(ui, x, y, w, h)
	local color_fill = theme.widget_bg
	local fill = false
	if ui.hover == self then
		color_fill = theme.widget_bg
		fill = true
	end
	if fill then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
	end
	tessera.graphics.set_font_size()
	tessera.graphics.set_color(self.text_color)
	tessera.graphics.label(self.text, x + self.indent, y, w - self.indent, h, self.align)
end

return Button
