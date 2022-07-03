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
	
	love.graphics.push()
		love.graphics.translate(BORDER_SIZE, HEADER + BORDER_SIZE)
		self:draw()
	love.graphics.pop()

	love.graphics.setColor(Theme.header)
	if self.box.focus then
		love.graphics.setColor(Theme.header_focus)
	end
	love.graphics.rectangle("fill", 0, 0, w, HEADER)

	love.graphics.setFont(font_main)
	love.graphics.setColor(Theme.ui_text)
	drawText(self.name, 0, 0, w, HEADER, "left")
end
function View:mousepressed() end
function View:mousereleased() end
function View:update() end
function View:wheelmoved() end

function View:getDimensions()
	return self.box.w - 2*BORDER_SIZE, self.box.h - HEADER - 2*BORDER_SIZE
end

function View:getMouse()
	return Mouse.x - (self.box.x + BORDER_SIZE), Mouse.y - (self.box.y + HEADER + BORDER_SIZE)
end

-----------------------------------------------

-- empty view
DefaultView =  View:derive("Default")


-- kind of annoying but this has to be at the bottom
require("views/channelview")
require("views/pannerview")
require("views/parameterview")
require("views/songview")
require("views/testpadview")


-- Views = {
-- 	DefaultView,
-- 	ParameterView,
-- 	PannerView,
-- }