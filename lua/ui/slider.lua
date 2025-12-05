local SliderValue = require("ui/slider_value")

local CORNER_RADIUS = 6

local Slider = {}
Slider.__index = Slider

function Slider.new(options)
	local self = setmetatable({}, Slider)

	self.value = SliderValue.new(options)
	self.drag_start = 0.0

	return self
end

function Slider:update(ui, target, key)
	local x, y, w, h = ui:next()
	local hit = ui:hitbox(self, x, y, w, h)

	local v = self.value:to_normal(target[key])

	local interact = false

	if mouse.button_pressed == 3 and hit then
		if target[key] ~= self.value.default then
			command.run_and_register(command.Change.new(target, key, self.value.default))
		end
	end

	if ui.active == self then
		mouse:set_relative(true)
		if mouse.button_pressed then
			self.drag_start = v
			self.prev_value = target[key]
			self.active = true
		end
		if mouse.drag then
			assert(w > 0)
			local scale = 0.7 / w
			local new_normalized = util.clamp(self.drag_start + scale * mouse.dx, 0, 1)
			self.new_value = self.value:from_normal(new_normalized)
			target[key] = self.new_value
			interact = true
		end
	end

	if self.active and mouse.button_released then
		self.active = false

		if self.new_value ~= self.prev_value then
			local c = command.Change.new(target, key, self.new_value)
			c.prev_value = self.prev_value
			command.register(c)
		end

		if mouse.drag then
			local ox, oy = ui.view:get_origin()
			mouse:set_position(ox + x + v * w, oy + y + 0.5 * h)
		end
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	local display = self.value:to_string(target[key])

	ui:push_draw(self.draw, { self, v, display, color_fill, color_line, x, y, w, h })

	return interact
end

function Slider:draw(v, display, color_fill, color_line, x, y, w, h)
	-- background fill
	tessera.graphics.set_color(color_fill)
	tessera.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)

	-- pop scissor
	local sx, sy, sw, sh = tessera.graphics.get_scissor()

	local gx, gy = x, y
	gx, gy = tessera.graphics.transform_point(gx, gy)

	if self.value.centered then
		local x1 = gx + w * 0.5
		local x2 = x1 + w * (v - 0.5)
		if x2 < x1 then
			x1, x2 = x2, x1
		end
		tessera.graphics.intersect_scissor(x1, gy, x2 - x1, h)
	else
		tessera.graphics.intersect_scissor(gx, gy, w * v, h)
	end

	tessera.graphics.set_color(theme.widget)
	tessera.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)

	-- push scissor
	tessera.graphics.set_scissor(sx, sy, sw, sh)

	tessera.graphics.set_color(color_line)
	tessera.graphics.rectangle("line", x, y, w, h, CORNER_RADIUS)

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(display, x, y + 1, w, h, tessera.graphics.ALIGN_CENTER)
end

return Slider
