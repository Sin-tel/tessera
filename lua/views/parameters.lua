ParameterView = View:derive("Parameters")

Group = {}

function Group:new(name)
	local new = {}
	setmetatable(new,self)
	self.__index = self


	new.name = name
	new.collapse = false
	new.sliders = {}
	new.y = {}
	for i = 1,5 do
		table.insert(new.sliders,  Slider:new("more sliders!"))
		table.insert(new.y,  i)
	end
	new.sliders[1].name = "a slider"
	new.sliders[2].name = "another slider"

	return new
end

function Group:draw(y,w,selected)
	y0 = UI_GRID*y

	local str = self.name
	if self.collapse then
		str = "> " .. str		
	else
		str = "V " .. str
	end
	love.graphics.setColor(Theme.ui_text)
	drawText(str, 0, y0, w, UI_GRID, "left")

	if not self.collapse then

		love.graphics.setColor(Theme.bg_nested)
		love.graphics.rectangle("fill", 0, y0+UI_GRID, w, (self.y[#self.y])*UI_GRID)

		for i,v in ipairs(self.sliders) do
			local mode = false
			if  v == selected then
				if self.action == "slider" then
					mode = "press"
				else
					mode = "hover"
				end
			end
			v:draw(y0 + self.y[i]*UI_GRID, w, mode)
		end
	end
end

function Group:getLength()
	if self.collapse then
		return 1
	end
	return #self.y + 1
end

function ParameterView:new()
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.groups = {}
	table.insert(new.groups, Group:new("first"))
	table.insert(new.groups, Group:new("second"))
	table.insert(new.groups, Group:new("third"))

	new.select = nil
	new.action = nil

	new.scroll = 0
	new.scroll_ = 0

	return new
end

function ParameterView:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.scroll = self.scroll + (self.scroll_ - self.scroll)*0.5
	if math.abs(self.scroll - self.scroll_) < 2 then
		self.scroll = self.scroll_
	end

	self.scroll = clamp(self.scroll, 0, self:getMaxScroll())
	

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
				index = math.floor((my + self.scroll)/UI_GRID)
			end

			local y = 0
			self.select = nil
			for i,v in ipairs(self.groups) do
				if not v.collapse then
					local get = v.sliders[index-y]
					if get and mx > get:getPad(w)-2 then
						self.select = get
						self.select_i = index
						break
					end
				end
				y = y + v:getLength()
			end
		end
	else
		self.select = nil
	end
end

function ParameterView:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if Mouse.button == 1 then
		if self.select then
			self.select:dragStart()

			love.mouse.setRelativeMode( true )
			self.action = "slider"
		else
			local index = math.floor((my + self.scroll)/UI_GRID)

			for i,v in ipairs(self.groups) do
				if index == 0 then
					v.collapse = not v.collapse
					break
				end
				index = index - v:getLength()
			end
		end
	end
end

function ParameterView:mousereleased()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if self.action == "slider" then
		local s = self.select:getPosition(w)
		love.mouse.setRelativeMode(false)
		if Mouse.drag then
			love.mouse.setPosition(self.box.x + s, self.box.y + self.select_i*UI_GRID + HEADER + 0.5*UI_GRID - self.scroll)
		end
	end
	self.action = nil
end

function ParameterView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	local y = -self.scroll/UI_GRID
	for k,group in ipairs(self.groups) do
		group:draw(y,w,self.select)
		y = y + group:getLength()
	end



	-- love.graphics.setColor(Theme.header)
	-- love.graphics.line(0, UI_GRID, w, UI_GRID)
	-- love.graphics.line(0, UI_GRID*2, w, UI_GRID*2)
	-- love.graphics.line(0, UI_GRID*3, w, UI_GRID*3)


end

function ParameterView:wheelmoved(y)
	self.scroll_ = math.floor(self.scroll - y*1.5*UI_GRID)

	self.scroll_ = clamp(self.scroll_, 0, self:getMaxScroll())
end

function ParameterView:getMaxScroll()
	local w, h = self:getDimensions()

	local l = 0
	for k,group in ipairs(self.groups) do
		l = l + group:getLength()
	end
	return math.max(0, (l*UI_GRID) - h)
end