local Ui = require("ui/ui")

local View = {}
View.__index = View

function View.new()
	local self = setmetatable({}, View)

	return self
end

function View.derive(name)
	local self = setmetatable({}, View)

	self.name = name

	-- dummy values
	self.w = 32
	self.h = 32
	return self
end

function View:draw() end

function View:drawFull()
	love.graphics.push()
	love.graphics.translate(Ui.BORDER_SIZE, Ui.HEADER + Ui.BORDER_SIZE)

	self.w = self.box.w - 2 * Ui.BORDER_SIZE
	self.h = self.box.h - Ui.HEADER - 2 * Ui.BORDER_SIZE
	self:draw()
	love.graphics.pop()

	love.graphics.setColor(theme.header)
	if self.box.focus then
		love.graphics.setColor(theme.header_focus)
	end
	love.graphics.rectangle("fill", 0, 0, self.box.w, Ui.HEADER)

	love.graphics.setFont(resources.fonts.main)
	love.graphics.setColor(theme.ui_text)

	util.drawText(self.name, 0, 0, self.box.w, Ui.HEADER, "left", true)
end

function View:mousepressed() end
function View:mousereleased() end
function View:mousereleased() end
function View:keypressed(key) end
function View:update() end

function View:getMouse()
	return mouse.x - (self.box.x + Ui.BORDER_SIZE), mouse.y - (self.box.y + Ui.HEADER + Ui.BORDER_SIZE)
end

function View:focus()
	return self.box.focus
end

function View:getOrigin()
	-- TODO: this should be in sync with the translate() calls in both box and view
	return self.box.x + Ui.BORDER_SIZE, self.box.y + Ui.HEADER + Ui.BORDER_SIZE
end

return View
