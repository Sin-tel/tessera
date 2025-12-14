local Ui = require("ui/ui")

local Toggle = {}
Toggle.__index = Toggle

function Toggle.new(target, key, options)
	local self = setmetatable({}, Toggle)

	self.no_undo = options.no_undo

	self.target = target
	self.key = key

	self.label = options.label
	self.style = options.style or "checkbox"

	self.pad = options.pad
	self.size = options.size or 1.0

	return self
end

function Toggle:update(ui)
	local x, y, w, h = ui:next()

	ui:hitbox(self, x, y, w, h)

	local clicked = false

	if ui.clicked == self then
		local new_v = true
		if self.target[self.key] then
			new_v = false
		end

		if self.no_undo then
			self.target[self.key] = new_v
		else
			command.run_and_register(command.Change.new(self.target, self.key, new_v))
		end

		clicked = true
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line

	if self.target[self.key] then
		color_fill = theme.widget
	end
	if ui.active == self then
		color_fill = theme.widget_press
	end
	local hover = ui.hover == self and ui.active ~= self
	if hover then
		color_line = theme.line_hover
	end

	if self.style == "checkbox" then
		ui:push_draw(self.draw_checkbox, { self, color_fill, color_line, x, y, w, h })
	elseif self.style == "menu" then
		ui:push_draw(self.draw_menu, { self, hover, color_fill, color_line, x, y, w, h })
	elseif self.style == "toggle" then
		ui:push_draw(self.draw_toggle, { self, color_fill, color_line, x, y, w, h })
	else
		error("unreachable")
	end

	return clicked
end

function Toggle:draw_toggle(color_fill, color_line, x, y, w, h)
	if w > 10 then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
		tessera.graphics.set_color(color_line)
		tessera.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
		if self.label then
			tessera.graphics.set_color(theme.ui_text)
			tessera.graphics.label(self.label, x, y, w, h, tessera.graphics.ALIGN_CENTER)
		end
	end
end

function Toggle:draw_checkbox(color_fill, color_line, x, y, w, h)
	local s = h * self.size
	local x1 = x + 0.5 * (h - s)
	local y1 = y + 0.5 * (h - s)

	tessera.graphics.set_color(color_fill)
	tessera.graphics.rectangle("fill", x1, y1, s, s, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(color_line)
	tessera.graphics.rectangle("line", x1, y1, s, s, Ui.CORNER_RADIUS)
	if self.label then
		tessera.graphics.set_color(theme.ui_text)
		local left_pad = self.pad or h + Ui.PAD
		tessera.graphics.label(self.label, x + left_pad, y, w - left_pad, h)
	end
end

function Toggle:draw_menu(hover, color_fill, color_line, x, y, w, h)
	if hover then
		tessera.graphics.set_color(theme.widget_bg)
		tessera.graphics.rectangle("fill", x, y, w, h)
	end

	local s = h * self.size
	local x1 = x + 0.5 * (h - s)
	local y1 = y + 0.5 * (h - s)

	tessera.graphics.set_color(color_fill)
	tessera.graphics.rectangle("fill", x1, y1, s, s, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(color_line)
	tessera.graphics.rectangle("line", x1, y1, s, s, Ui.CORNER_RADIUS)

	tessera.graphics.set_color(theme.ui_text)
	local left_pad = self.pad or h + Ui.PAD
	tessera.graphics.label(self.label, x + left_pad, y, w - left_pad, h)
end

return Toggle
