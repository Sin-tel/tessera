-- small gui framework
-- loosely inspired by egui (https://github.com/emilk/egui) and SUIT (https://github.com/vrld/suit/)
local mouse = require("mouse")
local Ui = {}
Ui.__index = Ui

-- TODO: we need to update this in a lot more places
local function scale(x)
	return math.floor(x * tessera.graphics.scale_factor + 0.5)
end
Ui.scale = scale

Ui.RESIZE_W = scale(5)
Ui.MIN_SIZE = scale(32)
Ui.HEADER = scale(32)
Ui.BORDER_RADIUS = scale(4)
Ui.BORDER_SIZE = scale(1)
Ui.RIBBON_HEIGHT = scale(32)
Ui.STATUS_HEIGHT = scale(20)

Ui.ROW_HEIGHT = scale(28)
Ui.PARAMETER_LABEL_WIDTH = scale(200) -- max width of parameter labels
Ui.PARAMETER_PAD = scale(8) -- padding for parameters
Ui.CORNER_RADIUS = scale(4)

Ui.PAD = scale(5)
Ui.TITLE_FONT_SIZE = scale(16)

function Ui.new(view)
	local self = setmetatable({}, Ui)

	self.draw_queue = {}
	self.view = view

	self.hover = false
	self.active = false
	self.clicked = false
	self.was_active = false

	self.bg_color = nil
	self.bg_list = {}

	self.scroll = 0
	self.scroll_goal = 0
	self.max_scroll = 0

	-- defer loading because of circular import
	local Layout = require("ui/layout")
	self.layout = Layout.new()

	return self
end

function Ui:start_frame(x, y)
	x = x or 0
	y = y or 0
	self.mx, self.my = self.view:get_mouse()

	self.bg_color = nil

	self.hover = false

	if self.clicked then
		self.clicked_prev = self.clicked
	end
	self.clicked = false
	self.double_click = false
	if mouse.button_released then
		self.was_active = self.active
		self.active = false
	end

	if mouse.scroll and self.view:focus() then
		self.scroll_goal = self.scroll_goal - 2 * mouse.scroll * self.ROW_HEIGHT
	end
	self.scroll_goal = util.clamp(self.scroll_goal, 0, self.max_scroll)
	self.scroll = util.lerp(self.scroll, self.scroll_goal, 0.3)
	self.scroll = util.towards(self.scroll, self.scroll_goal, 3.0)

	self.layout:start(x, y - self.scroll)
end

function Ui:end_frame()
	self.max_scroll = math.max(0, self.layout:total_height() - self.view.h)
end

function Ui:next(h)
	if not self.layout.ok then
		-- TODO: do col("max") when in column mode
		self.layout:row(self.view.w, h)
	end

	if self.bg_color then
		table.insert(self.bg_list, { self.layout.row_y, self.layout.row_h, self.bg_color })
	end

	return self.layout:get()
end

local function nothing() end

local function draw_label(text, align, color, x, y, w, h)
	tessera.graphics.set_color(color)
	tessera.graphics.label(text, x, y, w, h, align)
end

function Ui:label(text, align, color)
	local x, y, w, h = self:next()
	color = color or theme.ui_text
	self:push_draw(draw_label, { text, align, color, x, y, w, h })
end

function Ui:separator(text, align, color)
	self:next(self.layout.h * 0.5)
	self:push_draw(nothing, {})
end

function Ui:hitbox(widget, x, y, w, h)
	if
		self.view:focus()
		and self.mx >= x - 1
		and self.my >= y - 1
		and self.mx <= x + w + 2
		and self.my <= y + h + 2
	then
		if mouse.button_pressed == 1 and not self.active then
			-- cancel press for any overlapping elements
			-- mouse.button_pressed = nil
			self.active = widget
		end
		if (mouse.button_released == 1 and self.was_active == widget) and not self.clicked then
			self.clicked = widget

			if self.clicked_prev == self.clicked and mouse.double_click then
				self.double_click = self.clicked
			end
		end
		if (not self.active or self.active == widget) and not self.hover then
			self.hover = widget
		end

		return true
	end
	return false
end

function Ui:hit_area(x, y, w, h)
	return self.view:focus() and self.mx >= x and self.my >= y and self.mx <= x + w and self.my <= y + h
end

function Ui:background(color)
	self.bg_color = color
end

function Ui:push_draw(f, args)
	table.insert(self.draw_queue, { f, args })
end

function Ui:draw()
	for _, b in ipairs(self.bg_list) do
		tessera.graphics.set_color(b[3])
		tessera.graphics.rectangle("fill", 0, b[1], self.view.w, b[2])
	end

	for i in ipairs(self.draw_queue) do
		local f, args = unpack(self.draw_queue[i])
		f(unpack(args))
	end

	self.bg_list = {}
	self.draw_queue = {}
end

return Ui
