Workspace = {}

RESIZE_W = 5
MIN_SIZE = 32

function Workspace:load(w,h)
	self.w = w
	self.h = h 
	self.view = View:new()

	cursor_default = love.mouse.getSystemCursor("arrow")
	cursor_v = love.mouse.getSystemCursor("sizewe")
	cursor_h = love.mouse.getSystemCursor("sizens")
	cursor_size = love.mouse.getSystemCursor("sizeall")
	cursor_hand = love.mouse.getSystemCursor("hand")
	cursor_cross = love.mouse.getSystemCursor("crosshair")
	cursor_wait = love.mouse.getSystemCursor("wait")
	cursor_ibeam = love.mouse.getSystemCursor("ibeam")
end

function Workspace:resize(w,h) 
	self.w = w
	self.h = h 

	self.view:resize(0,0,w,h)
	self.view:resize(0,0,w,h) -- second time to satisfy constraints properly
end

function Workspace:draw()
	self.view:draw()
end

function Workspace:update()
	if Mouse.drag and self.dragDiv then
		-- print("aaaaa")
		self.dragDiv:set_split(Mouse.x,Mouse.y)
		-- self.dragDiv.r = self.dragDiv.r + 0.01


		-- self.dragDiv:resize()
	end
end

function Workspace:mousepressed()
	local div = self.view:get_divider(Mouse.x, Mouse.y)
	if div then
		self.dragDiv = div
	end
end

function Workspace:mousereleased()

	self.dragDiv = nil
end

function Workspace:hover(mx, my)
	local div = self.dragDiv or self.view:get_divider(mx, my)
	if div then
		if div.vertical then
			love.mouse.setCursor( cursor_v )
		else
			love.mouse.setCursor( cursor_h )
		end
	else
		love.mouse.setCursor( cursor_default )
	end
end

View = {}

function View:new(x,y,w,h)
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.x = x or 0
	new.y = y or 0
	new.w = w or width
	new.h = h or height

	new.color = {math.random(), math.random(), math.random()}
	
	return new	
end

function View:draw()
	if self.children then
		for i, v in ipairs(self.children) do
			v:draw()
		end
	else
		love.graphics.setColor(self.color)
		love.graphics.rectangle("fill", self.x + 2, self.y + 2, self.w-4, self.h-4, 2)
	end
	
end

function View:get_divider(mx, my)
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			if math.abs(self.r - mx) < RESIZE_W then
				return self
			else
				local d1 = self.children[1]:get_divider(mx, my)
				if d1 then return d1 end
				local d2 = self.children[2]:get_divider(mx - self.r, my) 
				if d2 then return d2 end
			end
		else
			if math.abs(self.r - my) < RESIZE_W then
				return self
			else
				local d1 = self.children[1]:get_divider(mx, my) 
				if d1 then return d1 end
				local d2 = self.children[2]:get_divider(mx, my - self.r) 
				if d2 then return d2 end
			end
		end
	end
	return false
end

function View:get(mx, my)
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			local d1 = self.children[1]:get(mx, my)
			if d1 then return d1 end
			local d2 = self.children[2]:get(mx - self.r, my) 
			if d2 then return d2 end
		else
			local d1 = self.children[1]:get(mx, my) 
			if d1 then return d1 end
			local d2 = self.children[2]:get(mx, my - self.r) 
			if d2 then return d2 end
		end
	end
	return self
end



function View:recalc(x,y,w,h)
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

function View:get_bound_left()
	if self.children then
		return math.max(self.children[1]:get_bound_left(), self.children[2]:get_bound_left())
	else
		return self.x
	end
end

function View:get_bound_right()
	if self.children then
		return math.min(self.children[1]:get_bound_right(), self.children[2]:get_bound_right())
	else
		return self.x + self.w
	end
end

function View:get_bound_up()
	if self.children then
		return math.max(self.children[1]:get_bound_up(), self.children[2]:get_bound_up())
	else
		return self.y
	end
end

function View:get_bound_down()
	if self.children then
		return math.min(self.children[1]:get_bound_down(), self.children[2]:get_bound_down())
	else
		return self.y + self.h
	end
end


function View:set_split(x,y)
	if not self.children then
		error("View has no div")
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

function View:split(r, vertical)
	if self.children then
		return false
	end
	self.children = {}
	self.vertical = vertical
	if vertical then
		self.r = self.w*r
		table.insert(self.children, View:new())
		table.insert(self.children, View:new())
	else
		self.r = self.h*r
		table.insert(self.children, View:new())
		table.insert(self.children, View:new())
	end

	self:recalc()
end

function View:resize(x,y,w,h)
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