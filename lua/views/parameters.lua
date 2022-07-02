ParameterView = View:derive("Parameters")

local Group = {}

function Group:new(name, paramlist)
	local new = {}
	setmetatable(new,self)
	self.__index = self


	new.name = name
	new.collapse = false
	new.sliders = {}

	for k,v in ipairs(paramlist) do
		table.insert(new.sliders, Slider:new(v))
	end 

	return new
end

function Group:draw(y,w,selected)
	y0 = UI_GRID*y

	local s = 32
	love.graphics.setColor(Theme.ui_text)
	if self.collapse then
		drawText(">", 0, y0, s, UI_GRID, "left")
	else
		drawText("V", 0, y0, s, UI_GRID, "left")
	end
	drawText("    " .. self.name, 0, y0, w - s, UI_GRID, "left")

	if not self.collapse then

		love.graphics.setColor(Theme.bg_nested)
		love.graphics.rectangle("fill", 0, y0+UI_GRID, w, (#self.sliders)*UI_GRID)

		for i,v in ipairs(self.sliders) do
			local mode = false
			if  v == selected then
				if self.action == "slider" then
					mode = "press"
				else
					mode = "hover"
				end
			end
			v:draw(y0 + i*UI_GRID, w, mode)
		end
	end
end


function Group:getLength()
	if self.collapse then
		return 1
	end
	return #self.sliders + 1
end

function ParameterView:makeparametergroups(channel)
	local parametergroups = {}
	table.insert(parametergroups, Group:new("channel", channel.channel))
	table.insert(parametergroups, Group:new(channel.instrument.name, channel.instrument.parameters))

	return parametergroups
end

function ParameterView:new()
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.groups = {}

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
	self.scroll = clamp(self.scroll, 0, self:getMaxScroll())
	self.scroll_ = clamp(self.scroll_, 0, self:getMaxScroll())


	if math.abs(self.scroll - self.scroll_) < 2 then
		self.scroll = self.scroll_
	end
	

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

	if selection.channel then
		self.groups = selection.channel.parametergroups
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
	elseif Mouse.button == 2 then
		if self.select then
			self.select:reset()
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
end

function ParameterView:wheelmoved(y)
	self.scroll_ = math.floor(self.scroll - y*1.5*UI_GRID)
end

function ParameterView:getMaxScroll()
	local w, h = self:getDimensions()

	local l = 0
	for k,group in ipairs(self.groups) do
		l = l + group:getLength()
	end
	return math.max(0, (l*UI_GRID) - h)
end