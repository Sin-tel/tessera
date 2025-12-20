local Ui = require("ui/ui")

-- stateless toggle button for use in channel widget

local ToggleSmall = {}
ToggleSmall.__index = ToggleSmall

function ToggleSmall.new(options)
	local self = setmetatable({}, ToggleSmall)

	self.label = options.label
	self.img_on = options.img_on
	self.img_off = options.img_off or options.img_on
	self.color_on = options.color_on or theme.ui_text
	self.color_off = options.color_off or theme.text_dim

	return self
end

function ToggleSmall:update(ui, checked)
	local x, y, w, h = ui:next()
	ui:hitbox(self, x, y, w, h)
	ui:push_draw(self.draw, { self, ui, checked, x, y, w, h })
	return ui.clicked == self
end

function ToggleSmall:draw(ui, checked, x, y, w, h)
	local color_fill
	if ui.hover == self and ui.active ~= self then
		color_fill = theme.bg_highlight
	end
	if color_fill then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
	end

	if checked then
		tessera.graphics.set_color(self.color_on)
	else
		tessera.graphics.set_color(self.color_off)
	end
	if self.label then
		tessera.graphics.set_font_size(21)
		local tw = tessera.graphics.measure_width(self.label)
		local o = 0.5 * (w - tw)
		tessera.graphics.text(self.label, x + o, y)
		tessera.graphics.set_font_size()
	else
		if checked then
			tessera.graphics.draw_path(self.img_on, x, y)
		else
			tessera.graphics.draw_path(self.img_off, x, y)
		end
	end
end

return ToggleSmall
