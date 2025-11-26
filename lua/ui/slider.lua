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

	local v = self.value:toNormal(target[key])

	local interact = false

	if mouse.button_pressed == 3 and hit then
		if target[key] ~= self.value.default then
			command.run_and_register(command.change.new(target, key, self.value.default))
		end
	end

	if ui.active == self then
		mouse:setRelative(true)
		if mouse.button_pressed then
			self.drag_start = v
			self.prev_value = target[key]
			self.active = true
		end
		if mouse.drag then
			assert(w > 0)
			local scale = 0.7 / w
			local new_normalized = util.clamp(self.drag_start + scale * mouse.dx, 0, 1)
			self.new_value = self.value:fromNormal(new_normalized)
			target[key] = self.new_value
			interact = true
		end
	end

	if self.active and mouse.button_released then
		self.active = false

		if self.new_value ~= self.prev_value then
			local c = command.change.new(target, key, self.new_value)
			c.prev_value = self.prev_value
			command.register(c)
		end

		if mouse.drag then
			local ox, oy = ui.view:getOrigin()
			mouse:setPosition(ox + x + v * w, oy + y + 0.5 * h)
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

	local display = self.value:toString(target[key])

	ui:pushDraw(self.draw, { self, v, display, color_fill, color_line, x, y, w, h })

	return interact
end

function Slider:draw(v, display, color_fill, color_line, x, y, w, h)
	-- background fill
	love.graphics.setColor(color_fill)
	love.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)

	-- pop scissor
	local sx, sy, sw, sh = love.graphics.getScissor()

	local gx, gy = x, y
	gx, gy = love.graphics.transformPoint(gx, gy)

	if self.value.centered then
		local x1 = gx + w * 0.5
		local x2 = x1 + w * (v - 0.5)
		if x2 < x1 then
			x1, x2 = x2, x1
		end
		love.graphics.intersectScissor(x1, gy, x2 - x1, h)
	else
		love.graphics.intersectScissor(gx, gy, w * v, h)
	end

	love.graphics.setColor(theme.widget)
	love.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)

	-- push scissor
	love.graphics.setScissor(sx, sy, sw, sh)

	love.graphics.setColor(color_line)
	love.graphics.rectangle("line", x, y, w, h, CORNER_RADIUS)

	love.graphics.setColor(theme.ui_text)
	util.drawText(display, x, y, w, h, "center")
end

return Slider
