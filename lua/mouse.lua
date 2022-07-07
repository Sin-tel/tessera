mouse = {}


DOUBLE_CLICK_TIME = 0.35
DRAG_DIST = 3


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

	self.cursor = cursor_default

	cursor_default = love.mouse.getSystemCursor("arrow")
	cursor_v = love.mouse.getSystemCursor("sizewe")
	cursor_h = love.mouse.getSystemCursor("sizens")
	cursor_size = love.mouse.getSystemCursor("sizeall")
	cursor_hand = love.mouse.getSystemCursor("hand")
	cursor_cross = love.mouse.getSystemCursor("crosshair")
	cursor_wait = love.mouse.getSystemCursor("wait")
	cursor_ibeam = love.mouse.getSystemCursor("ibeam")
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
		if math.sqrt((self.x - self.ix)^2 + (self.y - self.iy)^2) > DRAG_DIST then
			self.drag = true
		end
	end

	self.cursor = cursor_default
end

function mouse:updateCursor()
	if self.cursor then
		love.mouse.setVisible(true)
		love.mouse.setCursor( self.cursor )
	else
		love.mouse.setVisible(false)
	end
end

function mouse:mousemoved( x, y, dx, dy, istouch )
	if love.keyboard.isDown("lshift") then
		self.dx = self.dx + 0.1*dx
		self.dy = self.dy + 0.1*dy
	else
		self.dx = self.dx + dx
		self.dy = self.dy + dy
	end
end

