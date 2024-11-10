local Ui = require("ui/ui")

local Toggle = {}

function Toggle:new(text, options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.text = text
	new.style = options.style or "checkbox"

	return new
end

function Toggle:update(ui, target, key)
	local x, y, w, h = ui:next()

	ui:hitbox(self, x, y, w, h)

	local clicked = false

	if ui.clicked == self then
		command.run_and_register(command.change.new(target, key, not target[key]))
		clicked = true
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line

	if target[key] then
		color_fill = theme.widget
	end
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	if self.style == "checkbox" then
		ui:pushDraw(self.draw_checkbox, { self, color_fill, color_line, x, y, w, h })
	else
		ui:pushDraw(self.draw_toggle, { self, color_fill, color_line, x, y, w, h })
	end

	return clicked
end

function Toggle:draw_toggle(color_fill, color_line, x, y, w, h)
	if w > 10 then
		love.graphics.setColor(color_fill)
		love.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
		love.graphics.setColor(color_line)
		love.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
		love.graphics.setColor(theme.ui_text)
		util.drawText(self.text, x, y, w, h, "center", true)
	end
end

function Toggle:draw_checkbox(color_fill, color_line, x, y, w, h)
	love.graphics.setColor(color_fill)
	love.graphics.rectangle("fill", x, y, h, h, Ui.CORNER_RADIUS)
	love.graphics.setColor(color_line)
	love.graphics.rectangle("line", x, y, h, h, Ui.CORNER_RADIUS)
	love.graphics.setColor(theme.ui_text)
	local left_pad = h + Ui.DEFAULT_PAD
	util.drawText(self.text, x + left_pad, y, w - left_pad, h)
end

return Toggle
