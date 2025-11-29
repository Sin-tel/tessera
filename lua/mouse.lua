local mouse = {}

local DOUBLE_CLICK_TIME = 0.35
local DRAG_DIST = 3

mouse.DRAG_DIST = DRAG_DIST

function mouse:load()
	self.x = 0
	self.y = 0

	self.timer = 10000

	self.double = false
	self.drag = false
	self.relative = false

	self.ix = 0
	self.iy = 0

	self.dx = 0
	self.dy = 0

	self.button = false

	self.button_pressed = false
	self.button_released = false

	self.pbutton = false

	self.scroll = false

	self.cursor = "default"
	self.pcursor = self.cursor
end

function mouse:pressed(x, y, button)
	self.x, self.y = x, y

	if not self.button then -- ignore buttons while other is pressed
		self.ix = x
		self.iy = y
		self.dx = 0
		self.dy = 0
		self.drag = false
		self.button = button
		self.button_pressed = button
		self.is_down = true

		-- TODO: remove?
		workspace:mousepressed()
	end
end

function mouse:released(x, y, button)
	self.x, self.y = x, y
	if button == self.button then
		self.button_released = self.button
		self.is_down = false

		-- double click detect
		self.double = false
		local new_timer = tessera.timer.get_time()
		if new_timer - self.timer < DOUBLE_CLICK_TIME and self.pbutton == button then
			self.double = true
		end
		self.pbutton = button
		self.timer = new_timer

		-- TODO: remove?
		workspace:mousereleased()
		self.button = false
	end
end

function mouse:update(x, y)
	self.x, self.y = tessera.mouse.get_position()

	if self.button then
		if math.sqrt((self.x - self.ix) ^ 2 + (self.y - self.iy) ^ 2) > DRAG_DIST then
			self.drag = true
		end
	end

	self.cursor = "default"
end

function mouse:end_frame()
	self.button_pressed = false
	self.button_released = false
	self.scroll = false

	tessera.mouse.set_relative_mode(self.relative)
	if self.cursor and not self.relative then
		tessera.mouse.set_visible(true)
		if self.pcursor ~= self.cursor then
			tessera.mouse.set_cursor(self.cursor)
		end
		self.pcursor = self.cursor
	else
		tessera.mouse.set_visible(false)
	end
	self.relative = false
end

function mouse:set_cursor(c)
	self.cursor = c
end

function mouse:set_relative(r)
	-- TODO: some weird issue with sliders
	--   Dragging them and then right clicking does not
	--   register if the mouse hasn't moved since turning
	--   off relative mode.
	self.relative = true
end

function mouse:set_position(x, y)
	self.x = x
	self.y = y
	tessera.mouse.set_position(x, y)
end

function mouse:mousemoved(_, _, dx, dy)
	if modifier_keys.shift then
		self.dx = self.dx + 0.1 * dx
		self.dy = self.dy + 0.1 * dy
	else
		self.dx = self.dx + dx
		self.dy = self.dy + dy
	end
end

function mouse:wheelmoved(y)
	self.scroll = y
end

return mouse
