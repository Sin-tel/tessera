Workspace = {}

RESIZE_W = 5
MIN_SIZE = 32
HEADER = 32
BORDER_RADIUS = 4
BORDER_SIZE = 1
RIBBON = 32

function Workspace:load()
	self.w = width
	self.h = height
	self.box = Box:new(0, RIBBON, width, height - RIBBON)
end

function Workspace:resize(w,h) 
	self.w = w
	self.h = h 

	self.box:resize(0,RIBBON,w,h- RIBBON)
	self.box:resize(0,RIBBON,w,h- RIBBON) -- second time to satisfy constraints properly
end

function Workspace:draw()
	self.box:draw()
end

function Workspace:update()
	if self.dragDiv and Mouse.drag then
		self.dragDiv:set_split(Mouse.x,Mouse.y)
	else
		-- update
		self.box:update()

	end
	local div = self.dragDiv
	if not Mouse.isDown then
		div = div or self.box:getDivider()
		self.box:forAll(function(b) b.focus = false end)
	end
	if div then
		if div.vertical then
			Mouse.cursor = cursor_v
		else
			Mouse.cursor = cursor_h
		end
	else
		if not Mouse.isDown then
			b = self.box:get()
			if b then
				b.focus = true
			end
		end
	end
end

function Workspace:mousepressed()
	local div = false
	if Mouse.button == 1 then
		div = self.box:getDivider(Mouse.x, Mouse.y)
		if div then
			self.dragDiv = div
		end
	end

	if not div then
		self.box:mousepressed()
	end
end

function Workspace:mousereleased()
	self.dragDiv = nil
	self.box:mousereleased()
end

Box = {}

function Box:new(x,y,w,h)
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.x = x or 0
	new.y = y or 0
	new.w = w or width
	new.h = h or height

	return new	
end

function Box:stencil()
	love.graphics.rectangle("fill", self.x + BORDER_SIZE, self.y + BORDER_SIZE, self.w-(2*BORDER_SIZE), self.h-(2*BORDER_SIZE), BORDER_RADIUS)
end

function Box:update()
	if self.children then
		self.children[1]:update()
		self.children[2]:update()
	else
		self.view:update()
	end
end

function Box:mousepressed()
	if self.children then
		self.children[1]:mousepressed()
		self.children[2]:mousepressed()
	elseif self.focus then
		self.view:mousepressed()
	end
end

function Box:mousereleased()
	if self.children then
		self.children[1]:mousereleased()
		self.children[2]:mousereleased()
	elseif self.focus then
		self.view:mousereleased()
	end
end

function Box:forAll(f)
	f(self)
	if self.children then
		self.children[1]:forAll(f)
		self.children[2]:forAll(f)
	end
end

function Box:draw()
	if self.children then
		for i, v in ipairs(self.children) do
			v:draw()
		end
	else
		love.graphics.stencil( function() self:stencil() end, "replace", 2, true )
		love.graphics.setStencilTest("greater", 1)

		love.graphics.setColor(Theme.background)
		love.graphics.rectangle("fill", self.x, self.y, self.w, self.h)

		love.graphics.push()
			love.graphics.translate(self.x, self.y)
			self.view:drawFull()
		love.graphics.pop()

		love.graphics.setStencilTest()
		love.graphics.setColor(Theme.borders)
		love.graphics.rectangle("line", 
			self.x + BORDER_SIZE, self.y + BORDER_SIZE, 
			self.w-(2*BORDER_SIZE), self.h-(2*BORDER_SIZE), 
			BORDER_RADIUS)
	end
	
end

function Box:getDivider()
	local mx = Mouse.x - self.x
	local my = Mouse.y - self.y
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			if math.abs(self.r - mx) < RESIZE_W then
				return self
			else
				local d1 = self.children[1]:getDivider()
				if d1 then return d1 end
				local d2 = self.children[2]:getDivider() 
				if d2 then return d2 end
			end
		else
			if math.abs(self.r - my) < RESIZE_W then
				return self
			else
				local d1 = self.children[1]:getDivider() 
				if d1 then return d1 end
				local d2 = self.children[2]:getDivider()
				if d2 then return d2 end
			end
		end
	end
	return false
end

function Box:get()
	local mx = Mouse.x - self.x
	local my = Mouse.y - self.y
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			local d1 = self.children[1]:get()
			if d1 then return d1 end
			local d2 = self.children[2]:get()
			if d2 then return d2 end
		else
			local d1 = self.children[1]:get()
			if d1 then return d1 end
			local d2 = self.children[2]:get()
			if d2 then return d2 end
		end
	end
	return self
end



function Box:recalc(x,y,w,h)
	if x then
		if self.children then
			if self.vertical then
				self.r = self.r + (self.x - x)
			else
				self.r = self.r + (self.y - y)
			end
		end
		self.x = x
		self.y = y
		self.w = w
		self.h = h
	end

	self.x = math.floor(self.x)
	self.y = math.floor(self.y)
	self.w = math.floor(self.w)
	self.h = math.floor(self.h)

	if self.children then
		if self.vertical then
			self.children[1]:recalc(self.x, self.y, self.r         , self.h)
			self.children[2]:recalc(self.x + self.r, self.y, self.w - self.r, self.h)
		else
			self.children[1]:recalc(self.x, self.y, self.w, self.r   )
			self.children[2]:recalc(self.x, self.y + self.r, self.w, self.h - self.r)
		end
	end
end

function Box:get_bound_left()
	if self.children then
		return math.max(self.children[1]:get_bound_left(), self.children[2]:get_bound_left())
	else
		return self.x
	end
end

function Box:get_bound_right()
	if self.children then
		return math.min(self.children[1]:get_bound_right(), self.children[2]:get_bound_right())
	else
		return self.x + self.w
	end
end

function Box:get_bound_up()
	if self.children then
		return math.max(self.children[1]:get_bound_up(), self.children[2]:get_bound_up())
	else
		return self.y
	end
end

function Box:get_bound_down()
	if self.children then
		return math.min(self.children[1]:get_bound_down(), self.children[2]:get_bound_down())
	else
		return self.y + self.h
	end
end


function Box:set_split(x,y)
	if not self.children then
		error("Box has no div")
	end
	if self.vertical then
		self.r = (x - self.x) 
	else
		self.r = (y - self.y) 
	end

	if self.vertical then
		local bleft = self.children[1]:get_bound_left() - self.x + MIN_SIZE
		local bright = self.children[2]:get_bound_right() - self.x - MIN_SIZE
		self.r = math.min(math.max(self.r, bleft), bright)
	else
		local bup = self.children[1]:get_bound_up() - self.y + MIN_SIZE
		local bdown = self.children[2]:get_bound_down() - self.y - MIN_SIZE
		self.r = math.min(math.max(self.r, bup), bdown)
	end

	self:recalc()
end

function Box:split(r, vertical)
	if self.children then
		return false
	end
	self.children = {}
	self.vertical = vertical
	if vertical then
		self.r = self.w*r
		table.insert(self.children, Box:new())
		table.insert(self.children, Box:new())
	else
		self.r = self.h*r
		table.insert(self.children, Box:new())
		table.insert(self.children, Box:new())
	end

	self:recalc()
end

function Box:resize(x,y,w,h)
	if self.children then
		if self.vertical then
			self.r = w*self.r/self.w
		else
			self.r = h*self.r/self.h
		end
	end
	self.x = x
	self.y = y
	self.w = w
	self.h = h


	if self.children then

		if self.vertical then
			self.r = math.min(math.max(self.r, MIN_SIZE), self.w - MIN_SIZE)
		else
			self.r = math.min(math.max(self.r, MIN_SIZE), self.h - MIN_SIZE)
		end
		if self.vertical then
			self.children[1]:resize(self.x, self.y, self.r         , self.h)
			self.children[2]:resize(self.x + self.r, self.y, self.w - self.r, self.h)
		else
			self.children[1]:resize(self.x, self.y, self.w, self.r   )
			self.children[2]:resize(self.x, self.y + self.r, self.w, self.h - self.r)
		end

		self:set_split(self.x + self.r, self.y + self.r)
	end
end

function Box:setView(view)
	self.view = view
	view.box = self
end