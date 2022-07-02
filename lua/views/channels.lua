ChannelView =  View:derive("Channels")

function ChannelView:new()
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.select = nil

	new.scroll = 0
	new.scroll_ = 0

	return new
end

function ChannelView:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.scroll = self.scroll + (self.scroll_ - self.scroll)*0.5
	self.scroll = clamp(self.scroll, 0, self:getMaxScroll())
	self.scroll_ = clamp(self.scroll_, 0, self:getMaxScroll())


	if math.abs(self.scroll - self.scroll_) < 2 then
		self.scroll = self.scroll_
	end
	

	if self.box.focus then
		self.index = math.floor((my + self.scroll)/UI_GRID) + 1
	end
end

function ChannelView:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if Mouse.button == 1 then
		for i,v in ipairs(channels.list) do
			if i == self.index then
				selection.channel = v
			end
		end
	end
end

function ChannelView:mousereleased()

end

function ChannelView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	for i, v in ipairs(channels.list) do
		local y = UI_GRID*(i-1) - self.scroll

		if self.index == i then
			love.graphics.setColor(Theme.bg_highlight)
			love.graphics.rectangle("fill", 0, y, w, UI_GRID)
		end

		love.graphics.setColor(Theme.ui_text)
		if selection.channel == v then
			love.graphics.setColor(Theme.highlight)
		end
		drawText(v.name, 0, y, w, UI_GRID, "left")
	end
end

function ChannelView:wheelmoved(y)
	self.scroll_ = math.floor(self.scroll - y*1.5*UI_GRID)
end

function ChannelView:getMaxScroll()
	local w, h = self:getDimensions()

	local l = #channels.list
	return math.max(0, (l*UI_GRID) - h)
end