local Ui = require("ui/ui")

local Toggle = {}
Toggle.__index = Toggle

-- Note internal state is a float (0, 1) not a bool
-- use state == 1 for bool

function Toggle.new(text, options)
	local self = setmetatable({}, Toggle)

	self.no_undo = options.no_undo

	self.text = text
	self.style = options.style or "checkbox"

	self.pad = options.pad

	return self
end

function Toggle:update(ui, target, key)
	local x, y, w, h = ui:next()

	ui:hitbox(self, x, y, w, h)

	local clicked = false

	if ui.clicked == self then
		local new_v = 1
		if target[key] == 1 then
			new_v = 0
		end
		if self.no_undo then
			target[key] = new_v
		else
			command.run_and_register(command.Change.new(target, key, new_v))
		end

		clicked = true
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line

	if target[key] == 1 then
		color_fill = theme.widget
	end
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	if self.style == "checkbox" then
		ui:push_draw(self.draw_checkbox, { self, color_fill, color_line, x, y, w, h })
	else
		ui:push_draw(self.draw_toggle, { self, color_fill, color_line, x, y, w, h })
	end

	return clicked
end

function Toggle:draw_toggle(color_fill, color_line, x, y, w, h)
	if w > 10 then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
		tessera.graphics.set_color(color_line)
		tessera.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
		tessera.graphics.set_color(theme.ui_text)
		tessera.graphics.label(self.text, x, y, w, h, tessera.graphics.ALIGN_CENTER)
	end
end

function Toggle:draw_checkbox(color_fill, color_line, x, y, w, h)
	tessera.graphics.set_color(color_fill)
	tessera.graphics.rectangle("fill", x, y, h, h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(color_line)
	tessera.graphics.rectangle("line", x, y, h, h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(theme.ui_text)
	local left_pad = self.pad or h + Ui.PAD
	tessera.graphics.label(self.text, x + left_pad, y, w - left_pad, h)
end

return Toggle
