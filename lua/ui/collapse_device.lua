-- Container widget for device

local Ui = require("ui/ui")

local Toggle = {}
Toggle.__index = Toggle

local CollapseDevice = {}
CollapseDevice.__index = CollapseDevice

function CollapseDevice.new(device)
	local self = setmetatable({}, CollapseDevice)

	self.device = device
	self.toggle = Toggle.new(self.device)

	self.open = true
	self.angle = 0

	return self
end

function CollapseDevice:update(ui)
	local x, y, w, h = ui:next()

	local r = 0.25 * Ui.ROW_HEIGHT
	local bx = x + Ui.ROW_HEIGHT
	local by = y + h * 0.5
	self.toggle:update(ui, bx - r, by - r, 2 * r, 2 * r)

	ui:hitbox(self, x, y, w, h)

	if ui.clicked == self then
		self.open = not self.open
	end

	self.angle = 0.0
	if not self.open then
		self.angle = -0.5 * math.pi
	end

	ui:push_draw(self.draw, { self, x, y, w, h })

	return self.open
end

function CollapseDevice:draw(x, y, w, h)
	local tw = h * 0.15
	local cx, cy = x + h * 0.5, y + h * 0.5
	local x1, y1 = -tw, -tw
	local x2, y2 = tw, -tw
	local x3, y3 = 0, tw

	-- draw triangle
	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.push()
	tessera.graphics.translate(cx, cy)
	tessera.graphics.rotate(self.angle)
	tessera.graphics.polygon("fill", x1, y1, x2, y2, x3, y3)
	tessera.graphics.pop()

	-- draw label
	local label_pad = Ui.PAD + 1.25 * Ui.ROW_HEIGHT
	if self.device.data.mute then
		tessera.graphics.set_color(theme.text_dim)
	end
	tessera.graphics.set_font_size(Ui.TITLE_FONT_SIZE)
	tessera.graphics.label(self.device.data.display_name, x + label_pad, y, w - label_pad, h)
	tessera.graphics.set_font_size()

	-- draw meter
	local r = 0.2 * Ui.ROW_HEIGHT

	local ml = self.device.meter_l
	local mr = self.device.meter_r

	local cl = util.meter_color(ml, true)
	local cr = util.meter_color(mr, true)

	local mx_r = w - 1 * r - Ui.PAD
	local mx_l = w - 3 * r - 2 * Ui.PAD

	tessera.graphics.set_color(cl)
	tessera.graphics.circle("fill", mx_l, cy, r)
	tessera.graphics.set_color(cr)
	tessera.graphics.circle("fill", mx_r, cy, r)
end

-- mute toggle
function Toggle.new(device)
	local self = setmetatable({}, Toggle)

	self.device = device

	return self
end

function Toggle:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)

	local clicked = false

	if ui.clicked == self then
		self.device.data.mute = not self.device.data.mute
		clicked = true
	end

	local color_fill = theme.widget_bg
	local color_line = theme.line

	if not self.device.data.mute then
		color_fill = theme.widget
	end
	if ui.active == self then
		color_fill = theme.widget_press
	end
	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	ui:push_draw(self.draw, { self, color_fill, color_line, x, y, w, h })

	return clicked
end

function Toggle:draw(color_fill, color_line, x, y, w, h)
	tessera.graphics.set_color(color_fill)
	tessera.graphics.rectangle("fill", x, y, h, h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(color_line)
	tessera.graphics.rectangle("line", x, y, h, h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(theme.ui_text)
end

return CollapseDevice
