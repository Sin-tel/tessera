local Ui = require("ui/ui")
local util = require("util")
local command = require("command")

local Button = {}

-- TODO: middle mouse reset

local Selector = {}
function Selector:new(list, index)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.list = {}
	for _, v in ipairs(list) do
		table.insert(new.list, Button:new(v))
	end

	index = index or 1

	new.list[index].checked = true
	return new
end

function Selector:update(ui, target, key)
	local x, y, w, h = ui:next()

	local tx, ty = x, y
	local tw = w / #self.list

	local new_index = nil
	for i, v in ipairs(self.list) do
		if v:update(ui, tx, ty, tw, h) then
			if i ~= target[key] then
				new_index = i
			end
		end
		tx = tx + tw
	end

	if new_index then
		local c = command.change.new(target, key, new_index)
		c:run()
		command.register(c)
	end

	for i, v in ipairs(self.list) do
		v.checked = (i == target[key])
	end

	return new_index
end

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
	ui:pushDraw(self.draw, { self, ui, x, y, w, h })
	return ui.clicked == self
end

function Button:draw(ui, x, y, w, h)
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
		love.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
	end
	if color_line then
		love.graphics.setColor(color_line)
		love.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
	end

	love.graphics.setColor(theme.ui_text)
	util.drawText(self.text, x, y, w, h, "center", true)
end

return Selector
