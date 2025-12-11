-- TODO: replace this with an overlay of sorts

local Ui = require("ui/ui")

local Button = {}
Button.__index = Button

local Dropdown = {}
Dropdown.__index = Dropdown

function Dropdown.new(target, key, options)
	local self = setmetatable({}, Dropdown)

	self.title = options.title
	self.open = false

	self.target = target
	self.key = key

	self.no_state = target == nil
	self.no_undo = options.no_undo

	if self.no_state then
		assert(self.title)
	end

	self.list = {}
	for _, v in ipairs(options.list) do
		table.insert(self.list, Button.new(v))
	end

	return self
end

function Dropdown:update(ui)
	local x, y, w, h = ui:next()
	local new_index

	local label = self.title
	if not self.no_state then
		local index = self.target[self.key]
		label = self.list[index].text
	end

	ui:push_draw(self.draw, { self, ui, label, x, y, w, h })

	if self.open then
		local tx, ty = x, y
		local p = Ui.PAD
		local th = Ui.ROW_HEIGHT - 2 * p

		for i, v in ipairs(self.list) do
			ty = ty + th
			if v:update(ui, tx, ty, w, th) then
				new_index = i
			end
		end

		if new_index and not self.no_state then
			if self.no_undo then
				self.target[self.key] = new_index
			else
				command.run_and_register(command.Change.new(self.target, self.key, new_index))
			end
		end

		if mouse.button_released then
			self.open = false
		end
	else
		ui:hitbox(self, x, y, w, h)
		if ui.clicked == self then
			self.open = true
		end
	end

	return new_index
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
end

function Button.new(text)
	local self = setmetatable({}, Button)

	self.text = text

	return self
end

function Button:update(ui, x, y, w, h)
	ui:hitbox(self, x, y, w, h)
	ui:push_draw(self.draw, { self, ui, x, y, w, h })
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
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
	end
	if color_line then
		tessera.graphics.set_color(color_line)
		tessera.graphics.rectangle("line", x, y, w, h, Ui.CORNER_RADIUS)
	end

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(self.text, x, y, w, h, tessera.graphics.ALIGN_CENTER)
end

return Dropdown
