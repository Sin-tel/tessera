local SliderValue = require("ui/slider_value")
local Ui = require("ui/ui")

local CORNER_RADIUS = 6
-- Mouse travel in pixels that the value sticks at the notch while dragging.
local NOTCH_WIDTH = 24

local Slider = {}
Slider.__index = Slider

function Slider.new(target, key, options)
	local self = setmetatable({}, Slider)

	self.no_undo = options.no_undo
	-- Stick at the default while dragging past it. On by default for centered sliders.
	self.notch = options.notch
	if self.notch == nil then
		self.notch = options.centered
	end

	self.target = target
	self.key = key
	self.value = SliderValue.new(options)
	self.drag_start = 0.0
	-- background color, theme.widget_bg if nil
	self.color = nil

	return self
end

function Slider:update(ui)
	local x, y, w, h = ui:next()
	local hit = ui:hitbox(self, x, y, w, h)

	local v = self.value:to_normal(self.target[self.key])

	local value
	local commit = false
	local reset = false

	if (mouse.button_pressed == 3 or ui.double_click) and hit then
		reset = true
		if self.target[self.key] ~= self.value.default then
			if self.no_undo then
				self.target[self.key] = self.value.default
			else
				command.run_and_register(command.Change.new(self.target, self.key, self.value.default))
			end
			commit = true
		end
	end

	if ui.active == self then
		mouse:set_relative(true)
		-- Dragging moves along a track that is the normalized value with a gap of
		-- width `gap` inserted at the notch, where it holds still.
		assert(w > 0)
		local scale = 0.7 / w
		local notch, gap = self:notch_position(), 0
		if notch then
			gap = Ui.scale(NOTCH_WIDTH) * scale
		end
		if not self.active then
			self.drag_start = v
			if notch and math.abs(v - notch) < 1e-9 then
				self.drag_start = notch + 0.5 * gap
			elseif notch and v > notch then
				self.drag_start = v + gap
			end
			self.prev_value = self.target[self.key]
			self.active = true
		end
		if mouse.drag then
			local track = util.clamp(self.drag_start + scale * mouse.dx, 0, 1 + gap)
			local new_normalized = track
			if notch and track > notch then
				new_normalized = math.max(notch, track - gap)
			end
			self.new_value = self.value:from_normal(new_normalized)
			self.target[self.key] = self.new_value
			value = self.new_value
		end
	end

	if self.active and mouse.button_released then
		self.active = false
		if self.new_value and self.new_value ~= self.prev_value then
			if self.no_undo then
				self.target[self.key] = self.new_value
			else
				-- only commit on release
				local c = command.Change.new(self.target, self.key, self.new_value)
				c.prev_value = self.prev_value
				command.register(c)
			end
			commit = true
		end

		if mouse.drag then
			local ox, oy = ui.view:get_origin()
			mouse:set_position(ox + x + v * w, oy + y + 0.5 * h)
		end
	end

	local color_fill = self.color or theme.widget_bg
	local color_line = theme.line
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	local display = self.value:to_string(self.target[self.key])

	ui:push_draw(self.draw, { self, v, display, color_fill, color_line, x, y, w, h })

	-- value while dragging, commit when a change is done, reset on a middle or double click
	return value, commit, reset
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
	tessera.graphics.rectangle("fill", x - 0.5, y - 0.5, w, h + 1, CORNER_RADIUS)

	-- push scissor
	tessera.graphics.set_scissor(sx, sy, sw, sh)

	tessera.graphics.set_color(color_line)
	tessera.graphics.rectangle("line", x - 0.5, y - 0.5, w, h + 1, CORNER_RADIUS)

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(display, x, y + 0.4, w, h, tessera.graphics.ALIGN_CENTER)
end

-- Normalized value of the notch, nil if there is none.
function Slider:notch_position()
	if not self.notch then
		return nil
	end
	local n = self.value:to_normal(self.value.default)
	if n > 0 and n < 1 then
		return n
	end
end

function Slider:constrain()
	self.target[self.key] = self.value:constrain(self.target[self.key])
end

return Slider
