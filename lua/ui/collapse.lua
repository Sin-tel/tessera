local Ui = require("ui/ui")
local util = require("util")

local Collapse = {}

-- maybe this can share implementation with checkbox?
-- TODO: make a nice vector triangle instead of +/-

function Collapse:new(text)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.text = text
	new.open = true

	return new
end

function Collapse:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)

	if ui.clicked == self then
		self.open = not self.open
	end

	ui:pushDraw(self.draw, self, x, y, w, h)

	return self.open
end

function Collapse:draw(x, y, w, h)
	love.graphics.setColor(theme.ui_text)
	local left_pad = h + Ui.DEFAULT_PAD
	local text = "+"
	if self.open then
		text = "-"
	end
	util.drawText(text, x, y, h, h, "center")
	util.drawText(self.text, x + left_pad, y, w - left_pad, h)
end

return Collapse
