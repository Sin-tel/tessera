local SliderValue = require("ui/slider_value")
local command = require("command")

local Slider = {}

local CORNER_RADIUS = 6

function Slider:new(options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.value = SliderValue:new(options)
	new.drag_start = 0.0

	return new
end

function Slider:update(ui, target, key)
	local x, y, w, h = ui:next()
	local hit = ui:hitbox(self, x, y, w, h)

	local v = self.value:toNormal(target[key])

	local interact = false

	if mouse.button_pressed == 3 and hit then
		if target[key] ~= self.value.default then
			local c = command.change.new(target, key, self.value.default)
			c:run()
			command.register(c)
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

local function stencil(x, y, w, h)
	love.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)
end

function Slider:draw(v, display, color_fill, color_line, x, y, w, h)
	love.graphics.stencil(function()
		stencil(x, y, w, h)
	end, "increment", 1, true)
	love.graphics.setStencilTest("greater", 2)

	love.graphics.setColor(color_fill)
	love.graphics.rectangle("fill", x, y, w, h)
	love.graphics.setColor(theme.widget)
	if self.value.centered then
		love.graphics.rectangle("fill", x + w * 0.5, y, w * (v - 0.5), h)
	else
		love.graphics.rectangle("fill", x, y, w * v, h)
	end

	love.graphics.setStencilTest("greater", 1)

	love.graphics.setColor(color_line)
	love.graphics.rectangle("line", x, y, w, h, CORNER_RADIUS)

	love.graphics.setColor(theme.ui_text)
	util.drawText(display, x, y, w, h, "center")
end

return Slider
