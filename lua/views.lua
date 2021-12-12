

View = {}

function View:new()
	local new = {}
	setmetatable(new,self)
	self.__index = self
	
	return new	
end

function View:derive(name)
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.super = self
	new.name = name
	return new	
end


function View:draw() end

function View:drawFull()
	local w, h = self.box.w, self.box.h
	love.graphics.setColor(Theme.header)
	if self.box.focus then
		love.graphics.setColor(Theme.header_focus)
	end
	love.graphics.rectangle("fill", 0, 0, w, HEADER)
	
	love.graphics.setFont(font_main)
	love.graphics.setColor(Theme.header_text)
	drawText(self.name, 0, 0, w, HEADER, "left")
	love.graphics.push()
		love.graphics.translate(BORDER_SIZE, HEADER + BORDER_SIZE)
		self:draw()
	love.graphics.pop()
end
function View:mousepressed() end
function View:mousereleased() end
function View:update() end

function View:getDimensions()
	return self.box.w - 2*BORDER_SIZE, self.box.h - HEADER - 2*BORDER_SIZE
end

function View:getMouse()
	return Mouse.x - (self.box.x + BORDER_SIZE), Mouse.y - (self.box.y + HEADER + BORDER_SIZE)
end

-----------------------------------------------

DefaultView =  View:derive("Default")

TestView = View:derive("Test")
PannerView = View:derive("Panning")

function View:new()
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.sliders = {}
	new.y = {}
	for i = 1,5 do
		table.insert(new.sliders,  Slider:new("more sliders!"))
		table.insert(new.y,  i)
	end
	new.sliders[1].name = "a slider"
	new.sliders[2].name = "another slider"
	
	new.select = nil
	new.action = nil

	return new	
end

function TestView:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	-- self.select = nil

	if self.box.focus then
		local mx, my = self:getMouse()

		if self.action == "slider" then
			-- Mouse.cursor = nil
			if Mouse.drag then
				self.select:drag(w)
			end
		else
			local index = -1
			if self.box.focus then
				index = math.floor(my/UI_GRID)
			end
			if mx > self.sliders[1]:getPad(w)-2 then
				self.select = self.sliders[index]
			else
				self.select = nil
			end

			if self.select then
				self.select_i = index
				-- Mouse.cursor = cursor_v
			end
		end
	else
		self.select = nil
	end
end

function TestView:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if self.select then
		self.select:dragStart()

		love.mouse.setRelativeMode( true )
		self.action = "slider"
	end
end

function TestView:mousereleased()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if self.action == "slider" then
		local s = self.select:getPosition(w)

		love.mouse.setRelativeMode(false)
		if Mouse.drag then
			love.mouse.setPosition(self.box.x + s, self.box.y + self.y[self.select_i]*UI_GRID + HEADER + 0.5*UI_GRID)
		end
	end
	self.action = nil
end

function TestView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	love.graphics.setColor(Theme.ui_text)
	drawText("> Properties", 0, 0, w, UI_GRID, "left")


	love.graphics.setColor(Theme.header)
	love.graphics.line(0, UI_GRID, w, UI_GRID)
	love.graphics.line(0, UI_GRID*2, w, UI_GRID*2)
	love.graphics.line(0, UI_GRID*3, w, UI_GRID*3)

	for i,v in ipairs(self.sliders) do
		local mode = false
		if  v == self.select then
			if self.action == "slider" then
				mode = "press"
			else
				mode = "hover"
			end
		end
		v:draw(self.y[i]*UI_GRID, w, mode)
	end
end

function PannerView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()


	local radius = math.min(w/2,h)*0.95
	local radius2 = radius * 0.15

	if radius > 32 then

		local cx = w * 0.5
		local cy = h * 0.98

		mx = mx - cx
		my = my - cy
		local a = math.atan2(my, mx)

		local d = length(mx, my)
		d = clamp(d,radius2,radius)
		if a > math.pi*0.5 then a = -math.pi end
		a = clamp(a,-math.pi,0)

		mx = d*math.cos(a) + cx
		my = d*math.sin(a) + cy

		if self.box.focus then
			love.graphics.ellipse("fill", mx, my, 5)
			nd = (d-radius2) / (radius - radius2)


			nd = 17.31234*math.log(1-nd) -- magic formula

			love.graphics.print(string.format("%0.1f dB", (nd)), mx, my-24)
			-- love.graphics.print(string.format("%0.5f", from_dB(nd)), mx, my-24)
		end

		love.graphics.setColor(White)
		love.graphics.arc("line", "open", cx, cy, radius, 0, -math.pi)
		love.graphics.arc("line", "open", cx, cy, radius2, 0, -math.pi)
		love.graphics.line(cx - radius2, cy, cx - radius, cy)
		love.graphics.line(cx + radius2, cy, cx + radius, cy)
	end
end

Views = {
	DefaultView,
	TestView,
	PannerView,
}