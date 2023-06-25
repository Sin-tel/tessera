local ui = require("ui/ui")
local SliderValue = require("ui/slider_value")

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

function Slider:update(ui, x, y, w, h)
	local hit = ui:hitbox(self, x, y, w, h)

	local v = self.value:getNormalized()

	if mouse.button_pressed == 3 and hit then
		self.value:reset()
	end

	if ui.active == self then
		if mouse.button_pressed then
			self.drag_start = v
			self.active = true
			mouse:setRelative(true)
		end
		if mouse.drag then
			local scale = 0.7 / w
			local new_value = util.clamp(self.drag_start + scale * mouse.dx, 0, 1)
			self.value:setNormalized(new_value)
		end
	end

	if self.active and mouse.button_released then
		self.active = false
		if mouse.drag then
			local ox, oy = ui.view:getOrigin()
			mouse:setPosition(ox + x + v * w, oy + y + 0.5 * h)
		end
		mouse:setRelative(false)
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	ui:pushDraw(self.draw, self, color_fill, color_line, x, y, w, h)
end

local function stencil(x, y, w, h)
	love.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)
end

function Slider:draw(color_fill, color_line, x, y, w, h)
	local v = self.value:getNormalized()

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
	util.drawText(self.value:asString(), x, y, w, h, "center")
end

-- returns nil when not dirty
function Slider:getFloat()
	if self.value.dirty then
		self.value.dirty = false
		return self.value.v
	end
end

return Slider
