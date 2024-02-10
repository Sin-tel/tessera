local Ui = require("ui/ui")
local util = require("util")

local Collapse = {}

function Collapse:new(text)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.text = text
	new.open = true
	new.angle = 0

	return new
end

function Collapse:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)

	if ui.clicked == self then
		self.open = not self.open
	end

	-- local goal = 0.0
	-- if not self.open then
	-- 	goal = -0.5 * math.pi
	-- end
	-- self.angle = util.towards(self.angle, goal, 0.5)
	self.angle = 0.0
	if not self.open then
		self.angle = -0.5 * math.pi
	end

	ui:pushDraw(self.draw, { self, x, y, w, h })

	return self.open
end

function Collapse:draw(x, y, w, h)
	love.graphics.setColor(theme.ui_text)
	local left_pad = h * 0.8 + Ui.DEFAULT_PAD

	local tw = h * 0.15
	local cx, cy = x + h * 0.5, y + h * 0.5
	local x1, y1 = -tw, -tw
	local x2, y2 = tw, -tw
	local x3, y3 = 0, tw

	love.graphics.push()
	love.graphics.translate(cx, cy)
	love.graphics.rotate(self.angle)
	love.graphics.polygon("fill", x1, y1, x2, y2, x3, y3)
	love.graphics.polygon("line", x1, y1, x2, y2, x3, y3)
	love.graphics.pop()
	util.drawText(self.text, x + left_pad, y, w - left_pad, h)
end

return Collapse
