Mouse = {}

Mouse.x = 0
Mouse.y = 0

Mouse.double = false
Mouse.drag = false

Mouse.ix = 0
Mouse.iy = 0

Mouse.button = false

DOUBLE_CLICK_TIME = 0.35
DRAG_DIST = 3

function Mouse:pressed(x, y, button)
	self.x, self.y = x, y
	if not self.button then
		self.ix = x
		self.iy = y
		self.drag = false
		self.button = button

		Workspace:mousepressed()
	end
end

function Mouse:released(x, y, button)
	self.x, self.y = x, y
	if button == self.button then
		Workspace:mousereleased()
	end
	self.drag = false
	self.button = false
end

function Mouse:update()
	self.x, self.y = love.mouse.getPosition()

	if self.button then
		if math.sqrt((self.x - self.ix)^2 + (self.y - self.iy)^2) > DRAG_DIST then
			self.drag = true
		end
	end

	-- set cursor
	Workspace:hover(self.x, self.y)
end

