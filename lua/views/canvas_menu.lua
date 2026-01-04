local Ui = require("ui/ui")
local time = require("time")
local tuning = require("tuning")
local widgets = require("ui/widgets")

-- right click menu

local Menu = {}
Menu.__index = Menu

function Menu.new()
	local self = setmetatable({}, Menu)

	self.w = Ui.scale(260)
	self.h = 0 -- gets updated automatically

	self.x = mouse.x - 0.5 * self.w
	self.y = mouse.y - Ui.ROW_HEIGHT * 0.5

	self.x = math.max(0, self.x)
	self.y = math.max(0, self.y)

	self.ui = Ui.new(self)
	self.ui.layout:padding(4)

	-- widgets
	self.selector_time = widgets.Selector.new(project.settings, "snap_time", { list = time.snap_labels })
	self.selector_pitch = widgets.Selector.new(project.settings, "snap_pitch", { list = tuning.snap_labels })

	-- update once to pre-calculate layout
	self:update()
	self.ui:cancel_draw()

	return self
end

function Menu:update()
	-- local indent = Ui.scale(16)
	self.ui:start_frame(self.x, self.y)

	self.ui:label("Snap time")
	if self.selector_time:update(self.ui) then
		self.should_close = true
	end
	self.ui:label("Snap pitch")
	if self.selector_pitch:update(self.ui) then
		self.should_close = true
	end
	self.ui:end_frame()

	self.h = self.ui.layout:total_height()

	-- -- make sure it doesn't go off screen
	self.y = math.min(self.y, height - self.h)
end

function Menu:get_mouse()
	return mouse.x, mouse.y
end

function Menu:focus()
	return util.hit(self, mouse.x, mouse.y)
end

function Menu:draw()
	tessera.graphics.set_font_main()
	tessera.graphics.set_font_size()

	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", self.x, self.y, self.w, self.h, Ui.CORNER_RADIUS)

	self.ui:draw()
end

return Menu
