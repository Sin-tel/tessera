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

function View:draw_full()
	-- draw child window
	tessera.graphics.push()
	tessera.graphics.translate(Ui.BORDER_SIZE, Ui.HEADER + Ui.BORDER_SIZE)

	self:draw()
	tessera.graphics.pop()

	-- shadow
	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", 0, 0, self.box.w, Ui.HEADER + 2)

	-- header
	tessera.graphics.set_color(theme.header)
	if self.box.focus then
		tessera.graphics.set_color(theme.header_focus)
	end
	tessera.graphics.rectangle("fill", 0, 0, self.box.w, Ui.HEADER)

	-- title
	tessera.graphics.set_font_main()
	tessera.graphics.set_font_size(Ui.TITLE_FONT_SIZE)
	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(self.name, 10, 0, self.box.w - 20, Ui.HEADER)
	tessera.graphics.set_font_size()
end

function View:mousepressed() end
function View:mousereleased() end
function View:mousereleased() end
function View:keypressed(key) end
function View:update() end

function View:get_mouse()
	return mouse.x - (self.box.x + Ui.BORDER_SIZE), mouse.y - (self.box.y + Ui.HEADER + Ui.BORDER_SIZE)
end

function View:focus()
	return self.box.focus
end

function View:get_origin()
	-- TODO: this should be in sync with the translate() calls in both box and view
	return self.box.x + Ui.BORDER_SIZE, self.box.y + Ui.HEADER + Ui.BORDER_SIZE
end

function View:set_dimensions()
	self.w = self.box.w - 2 * Ui.BORDER_SIZE
	self.h = self.box.h - Ui.HEADER - 2 * Ui.BORDER_SIZE
end

return View
