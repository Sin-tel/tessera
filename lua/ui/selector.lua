local Ui = require("ui/ui")
local util = require("util")

local CORNER_RADIUS = 4
local Button = {}

function Button:new(text)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.text = text
	new.checked = false

	return new
end

function Button:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)
	ui:pushDraw(self.draw, self, ui, x, y, w, h)
	return ui.clicked == self
end

function Button:draw(ui, x, y, w, h)
	if w > 10 then
		local color_fill = nil
		local color_line = nil

		if ui.active == self then
			color_fill = theme.widget_press
		end
		if ui.hover == self and ui.active ~= self then
			color_line = theme.line_hover
		end
		if self.checked then
			color_fill = theme.widget
			color_line = theme.widget
		end
		if color_fill then
			love.graphics.setColor(color_fill)
			love.graphics.rectangle("fill", x, y, w, h, CORNER_RADIUS)
		end
		if color_line then
			love.graphics.setColor(color_line)
			love.graphics.rectangle("line", x, y, w, h, CORNER_RADIUS)
		end
		love.graphics.setColor(theme.ui_text)
		util.drawText(self.text, x, y, w, h, "center")
	end
end

local Selector = {}
function Selector:new(list)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.selected = 1

	new.list = {}
	for _, v in ipairs(list) do
		table.insert(new.list, Button:new(v))
	end

	new.list[new.selected].checked = true
	return new
end

function Selector:update(ui, x, y, w, h)
	local tx, ty = x, y
	local tw = w / #self.list

	local new_selected = nil
	for i, v in ipairs(self.list) do
		if v:update(ui, tx, ty, tw, h) then
			if i ~= self.selected then
				new_selected = i
			end
		end
		tx = tx + tw
	end

	if new_selected then
		self.selected = new_selected
		for i, v in ipairs(self.list) do
			v.checked = (i == self.selected)
		end
	end

	return self.selected
end

-- TODO: dirty flag
function Selector:getFloat()
	return self.selected - 1.0
end

return Selector
