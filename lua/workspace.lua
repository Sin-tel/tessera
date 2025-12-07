local Box = require("box")
local ui = require("ui/ui")
local views = require("views")

local workspace = {}

local Tab = {}
Tab.__index = Tab

function Tab.new(name)
	local self = setmetatable({}, Tab)

	self.name = name
	self.box = Box.new(0, ui.RIBBON_HEIGHT, width, height - ui.RIBBON_HEIGHT)
	return self
end

function Tab.from_data(data)
	assert(data.name)
	assert(data.box)
	local box = Box.from_data(data.box)
	return Tab.new(data.name, box)
end

function Tab:to_data()
	return {
		name = self.name,
		box = self.box:to_data(),
	}
end

function Tab:update_view()
	self.box:update_view()
end

function workspace:load()
	self.w = width
	self.h = height

	self.tab_current = 1
	self.tab_hover = nil

	self.cpu_load = 0
	self.meter = { l = 0, r = 0 }

	self.tabs = {}

	-- Load default workspace

	--- main
	local main = Tab.new("Main")

	local left, right = main.box:split(0.7, true)
	local top_left, middle_left = left:split(0.2, false)
	local top_right, bottom_rigth = right:split(0.35, false)

	top_left:set_view(views.Scope.new(false))
	middle_left:set_view(views.Canvas.new())

	-- top_left:set_view(views.Canvas.new())
	-- middle_left:set_view(views.TestPad.new())

	top_right:set_view(views.Channels.new())
	bottom_rigth:set_view(views.ChannelSettings.new())

	table.insert(self.tabs, main)

	--- settings
	local settings = Tab.new("Settings")
	settings.box:set_view(views.Settings.new())

	table.insert(self.tabs, settings)

	--- debug
	if not release then
		local debug_tab = Tab.new("Debug")
		local left2, right2 = debug_tab.box:split(0.5, true)
		left2:set_view(views.Debug.new())
		right2:set_view(views.UiTest.new())
		table.insert(self.tabs, debug_tab)
	end
end

function workspace:to_data()
	local data = {}

	data.tabs = {}
	for i, v in ipairs(self.tabs) do
		data.tabs[i] = v:to_data()
	end

	data.tab_current = self.tab_current

	return data
end

function workspace:switch_tab(prev)
	if prev then
		if self.tab_current == 1 then
			self.tab_current = #self.tabs
		else
			self.tab_current = self.tab_current - 1
		end
	else
		if self.tab_current == #self.tabs then
			self.tab_current = 1
		else
			self.tab_current = self.tab_current + 1
		end
	end
	-- refresh the new tab
	self:resize(self.w, self.h)
end

function workspace:resize(w, h)
	self.w = w
	self.h = h

	local box = self.tabs[self.tab_current].box
	local y = ui.RIBBON_HEIGHT
	local h2 = h - ui.RIBBON_HEIGHT - ui.STATUS_HEIGHT
	box:resize(0, y, w, h2)
	-- second time to satisfy constraints properly
	-- box:resize(0, y, w, h2)
end

function workspace:draw()
	tessera.graphics.set_font_size()
	-- menus
	-- TODO: just a mockup currently
	local sw = 60
	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label("File", 16, 2, sw, ui.RIBBON_HEIGHT, tessera.graphics.ALIGN_LEFT)
	tessera.graphics.label("Options", 16 + sw, 2, sw, ui.RIBBON_HEIGHT, tessera.graphics.ALIGN_LEFT)

	-- tabs
	-- TODO: just a mockup currently
	local st = 160

	local pad = ui.PAD
	tessera.graphics.set_font_size()
	for i, v in ipairs(self.tabs) do
		if i == self.tab_current then
			tessera.graphics.set_color(theme.header)
		elseif i == self.tab_hover then
			tessera.graphics.set_color(theme.bg_highlight)
		else
			tessera.graphics.set_color(theme.background)
		end
		tessera.graphics.rectangle("fill", st * i, pad, st - pad, ui.RIBBON_HEIGHT + 10, ui.BORDER_RADIUS)

		tessera.graphics.set_color(theme.ui_text)
		tessera.graphics.label(v.name, st * i, 2, st - pad, ui.RIBBON_HEIGHT, tessera.graphics.ALIGN_CENTER)
	end

	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", st * (#self.tabs + 1), pad, 32, ui.RIBBON_HEIGHT + 10, ui.BORDER_RADIUS)
	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label("+", st * (#self.tabs + 1), 2, 32, ui.RIBBON_HEIGHT, tessera.graphics.ALIGN_CENTER)

	-- CPU meter
	local ll = util.clamp(self.cpu_load, 0.01, 1)
	local hl_col = theme.cpu_meter
	if self.cpu_load > 1.0 then
		hl_col = theme.warning
	end

	local w1 = 64
	local h1 = 16
	local y1 = 0.5 * (ui.RIBBON_HEIGHT - h1)
	local x1 = self.w - 64 - y1

	tessera.graphics.set_color(theme.widget_bg)
	tessera.graphics.rectangle("fill", x1, y1, w1, h1, 2)
	tessera.graphics.set_color(hl_col)
	tessera.graphics.rectangle("fill", x1, y1, w1 * ll, h1)
	tessera.graphics.set_color(theme.line)
	tessera.graphics.rectangle("line", x1, y1, w1, h1, 2)

	tessera.graphics.set_color(theme.ui_text)
	local cpu_label = "offline"
	if tessera.audio.ok() then
		cpu_label = string.format("%d %%", 100 * self.cpu_load)
	end
	tessera.graphics.label(cpu_label, x1, 0, w1, ui.RIBBON_HEIGHT, tessera.graphics.ALIGN_CENTER)
	tessera.graphics.label("CPU: ", x1 - w1, 0, w1, ui.RIBBON_HEIGHT, tessera.graphics.ALIGN_RIGHT)

	-- master meters

	w1 = 96
	h1 = 16
	y1 = 0.5 * (ui.RIBBON_HEIGHT - h1)
	x1 = self.w - 224 - y1

	local ml = self.meter.l
	local mr = self.meter.r

	local wl = util.clamp((util.to_dB(ml) + 80) / 80, 0, 1)
	local wr = util.clamp((util.to_dB(mr) + 80) / 80, 0, 1)

	local cl = util.meter_color(ml)
	local cr = util.meter_color(mr)

	tessera.graphics.set_color(theme.bg_nested)
	tessera.graphics.rectangle("fill", x1, y1, w1, h1, 2)
	if wl > 0 then
		tessera.graphics.set_color(cl)
		tessera.graphics.rectangle("fill", x1, y1, w1 * wl, 0.5 * h1 - 1)
	end
	if wr > 0 then
		tessera.graphics.set_color(cr)
		tessera.graphics.rectangle("fill", x1, y1 + 0.5 * h1, w1 * wr, 0.5 * h1 - 1)
	end
	tessera.graphics.set_color(theme.line)
	tessera.graphics.rectangle("line", x1, y1 - 0.5, w1, h1, 2)
	tessera.graphics.set_color(theme.ui_text)

	self.tabs[self.tab_current].box:draw()

	-- status bar
	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", 0, self.h - ui.STATUS_HEIGHT, self.w, ui.STATUS_HEIGHT)
	tessera.graphics.set_color(theme.borders)
	tessera.graphics.line(0, self.h - ui.STATUS_HEIGHT, self.w, self.h - ui.STATUS_HEIGHT)
end

function workspace:update()
	if self.drag_div and mouse.drag then
		self.drag_div:set_split(mouse.x, mouse.y)
	end

	-- update active tab
	local tab = self.tabs[self.tab_current]
	tab.box:update_view()

	local div = self.drag_div
	if not mouse.is_down then
		div = div or tab.box:get_divider()
		tab.box:set_focus(false)
		self.focus = nil
	end
	if div then
		if div.vertical then
			mouse:set_cursor("v")
		else
			mouse:set_cursor("h")
		end
	else
		if not mouse.is_down then
			local b = tab.box:get()
			if b then
				b.focus = true
				self.focus = b
			end
		end
	end
end

function workspace:mousepressed()
	local div = false
	if mouse.button == 1 then
		local tab = self.tabs[self.tab_current]
		div = tab.box:get_divider(mouse.x, mouse.y)
		if div then
			self.drag_div = div
		end
	end

	if not div and self.focus then
		self.focus.view:mousepressed()
	end
end

function workspace:mousereleased()
	self.drag_div = nil
	if self.focus then
		self.focus.view:mousereleased()
	end
end

function workspace:keypressed(key)
	if self.focus then
		return self.focus.view:keypressed(key)
	end
end

return workspace
