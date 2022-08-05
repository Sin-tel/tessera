View = {}

function View:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	return new
end

function View:derive(name)
	local new = {}
	setmetatable(new, self)
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

	love.graphics.setColor(theme.header)
	if self.box.focus then
		love.graphics.setColor(theme.header_focus)
	end
	love.graphics.rectangle("fill", 0, 0, w, HEADER)

	love.graphics.setFont(fonts.main)
	love.graphics.setColor(theme.ui_text)
	drawText(self.name, 0, 0, w, HEADER, "left")
end
function View:mousepressed() end
function View:mousereleased() end
function View:update() end
function View:wheelmoved() end

function View:getDimensions()
	return self.box.w - 2 * BORDER_SIZE, self.box.h - HEADER - 2 * BORDER_SIZE
end

function View:getMouse()
	return mouse.x - (self.box.x + BORDER_SIZE), mouse.y - (self.box.y + HEADER + BORDER_SIZE)
end

-----------------------------------------------

-- empty view
DefaultView = View:derive("Default")

-- These have to be at the bottom of this file.
require("views/channelview")
require("views/pannerView")
require("views/parameterview")
require("views/songview")
require("views/testpadview")
require("views/scopeview")

-- Views = {
-- 	DefaultView,
-- 	parameterView,
-- 	pannerView,
-- }
