local mouse = {}

local DOUBLE_CLICK_TIME = 0.35
local DRAG_DIST = 3

function mouse:load()
	self.x = 0
	self.y = 0

	self.timer = 10000

	self.double = false
	self.drag = false

	self.ix = 0
	self.iy = 0

	self.dx = 0
	self.dy = 0

	self.button = false

	self.pbutton = false

	self.cursors = {}

	self.cursors.default = love.mouse.getSystemCursor("arrow")
	self.cursors.v = love.mouse.getSystemCursor("sizewe")
	self.cursors.h = love.mouse.getSystemCursor("sizens")
	self.cursors.size = love.mouse.getSystemCursor("sizeall")
	self.cursors.hand = love.mouse.getSystemCursor("hand")
	self.cursors.cross = love.mouse.getSystemCursor("crosshair")
	self.cursors.wait = love.mouse.getSystemCursor("wait")
	self.cursors.ibeam = love.mouse.getSystemCursor("ibeam")

	self.cursor = self.cursors.default
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

		-- double click detect
		self.double = false
		local newt = love.timer.getTime()
		if newt - self.timer < DOUBLE_CLICK_TIME and self.pbutton == button then
			self.double = true
		end
		self.pbutton = button
		self.timer = newt
		self.isDown = true

		-- handle press
		workspace:mousepressed()
	end
end

function mouse:released(x, y, button)
	self.x, self.y = x, y
	if button == self.button then
		self.isDown = false
		workspace:mousereleased()
		self.drag = false
		self.button = false
	end
end

function mouse:update(x, y)
	self.x, self.y = love.mouse.getPosition()
	self.x = x or self.x
	self.y = y or self.y

	if self.button then
		if math.sqrt((self.x - self.ix) ^ 2 + (self.y - self.iy) ^ 2) > DRAG_DIST then
			self.drag = true
		end
	end

	self.cursor = self.cursors.default
end

function mouse:updateCursor()
	if self.cursor then
		love.mouse.setVisible(true)
		love.mouse.setCursor(self.cursor)
	else
		love.mouse.setVisible(false)
	end
end

function mouse:setCursor(c)
	self.cursor = self.cursors[c]
end

function mouse:mousemoved(_, _, dx, dy)
	if love.keyboard.isDown("lshift") then
		self.dx = self.dx + 0.1 * dx
		self.dy = self.dy + 0.1 * dy
	else
		self.dx = self.dx + dx
		self.dy = self.dy + dy
	end
end

return mouse
