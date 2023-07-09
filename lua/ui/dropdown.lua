local Ui = require("ui/ui")
local util = require("util")

local Button = {}

local Dropdown = {}
function Dropdown:new(options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.title = options.title
	new.index = options.index or 1

	new.open = false

	new.list = {}
	for _, v in ipairs(options.list) do
		table.insert(new.list, Button:new(v))
	end

	new.list[new.index].checked = true
	return new
end

function Dropdown:update(ui, x, y, w, h)
	local new_index
	if self.open then
		local hover_any = false
		local tx, ty = x, y
		local p = Ui.DEFAULT_PAD
		local th = Ui.ROW_HEIGHT - 2 * p

		for i, v in ipairs(self.list) do
			ty = ty + th
			if v:update(ui, tx, ty, w, th) then
				new_index = i
			end
			if ui.hover == v then
				hover_any = true
			end
		end

		if new_index then
			self.index = new_index
			for i, v in ipairs(self.list) do
				v.checked = (i == self.index)
			end
			self.open = false
		end
		if mouse.button_pressed and not hover_any then
			self.open = false
		end
	else
		ui:hitbox(self, x, y, w, h)
		if ui.clicked == self then
			self.open = true
		end
	end

	ui:pushDraw(self.draw, self, ui, x, y, w, h)

	return new_index
end

function Dropdown:draw(ui, x, y, w, h)
	local color_fill = theme.widget_bg
	local color_line = theme.line

	local th = h

	if self.open then
		local p = Ui.DEFAULT_PAD
		local n = #self.list
		th = h + n * (Ui.ROW_HEIGHT - 2 * p)
	end

	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	if color_fill then
		love.graphics.setColor(color_fill)
		love.graphics.rectangle("fill", x, y, w, th, Ui.CORNER_RADIUS)
	end
	if color_line then
		love.graphics.setColor(color_line)
		love.graphics.rectangle("line", x, y, w, th, Ui.CORNER_RADIUS)
	end

	love.graphics.setColor(theme.ui_text)
	if self.title then
		util.drawText(self.title, x, y, w, h, "center", true)
	else
		util.drawText(self.list[self.index].text, x, y, w, h, "center", true)
	end
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
	ui:pushDraw(self.draw, self, ui, x, y, w, h)
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

return Dropdown
