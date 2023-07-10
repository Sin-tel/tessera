-- small gui framework
-- loosely inspired by egui (https://github.com/emilk/egui) and SUIT (https://github.com/vrld/suit/)
local mouse = require("mouse")
local Ui = {}

Ui.RESIZE_W = 5
Ui.MIN_SIZE = 32
Ui.HEADER = 32
Ui.BORDER_RADIUS = 4
Ui.BORDER_SIZE = 1
Ui.RIBBON_HEIGHT = 32

Ui.ROW_HEIGHT = 28
Ui.PARAMETER_LABEL_WIDTH = 150 -- max width of parameter labels
Ui.PARAMETER_PAD = 8 -- padding for parameters
Ui.BUTTON_SMALL = 18
Ui.CORNER_RADIUS = 4

Ui.DEFAULT_PAD = 5

function Ui:new(view)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.draw_queue = {}
	new.view = view

	new.hover = false
	new.active = false
	new.clicked = false
	new.was_active = false

	new.bg_color = nil
	new.bg_list = {}

	new.scroll = 0
	new.scroll_goal = 0
	new.max_scroll = 0

	local Layout = require("ui/layout")
	new.layout = Layout:new()

	return new
end

function Ui:startFrame()
	self.mx, self.my = self.view:getMouse()

	self.bg_color = nil

	self.hover = false
	self.clicked = false
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

	self.layout:start(0, -self.scroll)
end

function Ui:endFrame()
	local w, h = self.view:getDimensions()
	self.max_scroll = math.max(0, self.layout:totalHeight() - h)
end

function Ui:next()
	if not self.layout.ok then
		-- TODO: do col("max") when in column mode
		local w, h = self.view:getDimensions()
		self.layout:row(w, Ui.ROW_HEIGHT)
	end

	if self.bg_color then
		table.insert(self.bg_list, { self.layout.row_y, self.layout.row_h, self.bg_color })
	end
end

function Ui:put(widget)
	self:next()
	local ret = widget:update(self, self.layout:get())
	return ret
end

local function drawLabel(text, align, x, y, w, h)
	love.graphics.setColor(theme.ui_text)
	util.drawText(text, x, y, w, h, align)
end

function Ui:label(text, align)
	self:next()
	self:pushDraw(drawLabel, text, align, self.layout:get())
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
			self.active = widget
		end
		if (mouse.button_released == 1 and self.was_active == widget) and not self.clicked then
			self.clicked = widget
		end
		if (not self.active or self.active == widget) and not self.hover then
			self.hover = widget
		end

		return true
	end
	return false
end

function Ui:hitArea(x, y, w, h)
	return self.view:focus() and self.mx >= x and self.my >= y and self.mx <= x + w and self.my <= y + h
end

function Ui:background(color)
	self.bg_color = color
end

function Ui:pushDraw(f, ...)
	local args = { ... }
	table.insert(self.draw_queue, function()
		f(unpack(args))
	end)
end

function Ui:draw()
	local w, h = self.view:getDimensions()

	for _, b in ipairs(self.bg_list) do
		love.graphics.setColor(b[3])
		love.graphics.rectangle("fill", 0, b[1], w, b[2])
	end

	-- draw in reverse order to handle overlaps
	for i = #self.draw_queue, 1, -1 do
		self.draw_queue[i]()
	end

	-- TODO: maybe we can cache these?
	self.bg_list = {}
	self.draw_queue = {}
end

return Ui
