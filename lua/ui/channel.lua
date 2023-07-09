local Ui = require("ui/ui")
local util = require("util")

local Channel = {}

function Channel:new(channel)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.channel = channel

	return new
end

function Channel:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)

	ui:pushDraw(self.draw, self, ui, x, y, w, h)

	if ui.clicked == self then
		selection.channel = self.channel
	end

	return ui.clicked == self
end

function Channel:draw(ui, x, y, w, h)
	local color_fill = nil
	if ui.hover == self and ui.active ~= self then
		love.graphics.setColor(theme.bg_highlight)
		love.graphics.rectangle("fill", x, y, w, h)
	end

	love.graphics.setColor(theme.ui_text)
	if selection.channel == self.channel then
		love.graphics.setColor(theme.highlight)
	end

	util.drawText(self.channel.name, x, y, w, h, "left", true)
end

return Channel
