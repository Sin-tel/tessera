

ParameterView = View:derive("Parameters")
function ParameterView:new()
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

function ParameterView:update()
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

function ParameterView:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if self.select and Mouse.button == 1 then
		self.select:dragStart()

		love.mouse.setRelativeMode( true )
		self.action = "slider"
	end
end

function ParameterView:mousereleased()
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

function ParameterView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	love.graphics.setColor(Theme.ui_text)
	drawText("V Properties", 0, 0, w, UI_GRID, "left")


	love.graphics.setColor(Theme.header)
	love.graphics.line(0, UI_GRID, w, UI_GRID)
	-- love.graphics.line(0, UI_GRID*2, w, UI_GRID*2)
	-- love.graphics.line(0, UI_GRID*3, w, UI_GRID*3)

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
