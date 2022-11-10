local ui = require("ui")

local View = {}

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
	love.graphics.translate(ui.BORDER_SIZE, ui.HEADER + ui.BORDER_SIZE)
	self:draw()
	love.graphics.pop()

	love.graphics.setColor(theme.header)
	if self.box.focus then
		love.graphics.setColor(theme.header_focus)
	end
	love.graphics.rectangle("fill", 0, 0, w, ui.HEADER)

	love.graphics.setFont(resources.fonts.main)
	love.graphics.setColor(theme.ui_text)
	util.drawText(self.name, 0, 0, w, ui.HEADER, "left")
end
function View:mousepressed() end
function View:mousereleased() end
function View:update() end
function View:wheelmoved() end

function View:getDimensions()
	return self.box.w - 2 * ui.BORDER_SIZE, self.box.h - ui.HEADER - 2 * ui.BORDER_SIZE
end

function View:getMouse()
	return mouse.x - (self.box.x + ui.BORDER_SIZE), mouse.y - (self.box.y + ui.HEADER + ui.BORDER_SIZE)
end

return View
