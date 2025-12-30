local Ui = require("ui/ui")

local Dropdown = {}
Dropdown.__index = Dropdown

function Dropdown.new(target, key, options)
	local self = setmetatable({}, Dropdown)

	self.title = options.title
	self.open = false

	self.target = target
	self.key = key

	self.no_undo = options.no_undo

	if self.no_state then
		assert(self.title)
	end

	self.list = options.list

	self.arrows = options.arrows

	return self
end

function Dropdown:constrain_index(index)
	return (index - 1) % #self.list + 1
end

function Dropdown:update(ui)
	local x, y, w, h = ui:next()
	ui:hitbox(self, x, y, w, h)

	local hit = false

	if self.new_index and not self.no_state then
		if self.no_undo then
			self.target[self.key] = self.new_index
		else
			command.run_and_register(command.Change.new(self.target, self.key, self.new_index))
		end
		hit = self.new_index
		self.new_index = nil
	end

	local label = self.title
	if not self.title then
		local index = self.target[self.key]
		label = self.list[index]
	end

	ui:push_draw(self.draw, { self, ui, label, x, y, w, h })

	if ui.clicked == self then
		local x2 = ui.mx - x
		if self.arrows and x2 < w * 0.2 then
			self.new_index = self:constrain_index(self.target[self.key] - 1)
		elseif self.arrows and x2 > w * 0.8 then
			self.new_index = self:constrain_index(self.target[self.key] + 1)
		else
			workspace:set_overlay(self:menu())
		end
	end

	return hit
end

function Dropdown:draw(ui, label, x, y, w, h)
	local color_fill = theme.widget_bg
	local color_line = theme.line

	local th = h

	if self.open then
		local p = Ui.PAD
		local n = #self.list
		th = h + n * (Ui.ROW_HEIGHT - 2 * p)
	end

	if ui.hover == self and ui.active ~= self then
		color_line = theme.line_hover
	end

	if color_fill then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, th, Ui.CORNER_RADIUS)
	end
	if color_line then
		tessera.graphics.set_color(color_line)
		tessera.graphics.rectangle("line", x, y, w, th, Ui.CORNER_RADIUS)
	end

	if self.open then
		tessera.graphics.set_color(theme.text_tip)
	else
		tessera.graphics.set_color(theme.ui_text)
	end
	tessera.graphics.label(label, x, y, w, h, tessera.graphics.ALIGN_CENTER)

	if self.arrows then
		local tw = h * 0.15
		local cx, cy = x + h * 0.7, y + h * 0.5
		local x1, y1 = tw, -tw
		local x2, y2 = tw, tw
		local x3, y3 = -tw, 0

		tessera.graphics.push()
		tessera.graphics.translate(cx, cy)
		tessera.graphics.polygon("fill", x1, y1, x2, y2, x3, y3)
		tessera.graphics.pop()

		cx, cy = x + w - h * 0.7, y + h * 0.5
		x1, y1 = -tw, tw
		x2, y2 = -tw, -tw
		x3, y3 = tw, 0

		tessera.graphics.push()
		tessera.graphics.translate(cx, cy)
		tessera.graphics.polygon("fill", x1, y1, x2, y2, x3, y3)
		tessera.graphics.pop()
	end
end

function Dropdown:menu()
	local Menu = require("menu")
	local Button = require("ui/button")

	local options = {
		style = "menu",
		align = tessera.graphics.ALIGN_LEFT,
	}

	local items = {}
	for i, v in ipairs(self.list) do
		table.insert(items, {
			widget = Button.new(v, options),
			action = function()
				self.new_index = i
			end,
		})
	end

	return Menu.new(items)
end

return Dropdown
