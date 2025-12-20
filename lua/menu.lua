local Canvas = require("views/canvas")
local Ui = require("ui/ui")
local engine = require("engine")
local file = require("file")
local widgets = require("ui/widgets")

local Menu = {}
Menu.__index = Menu

local indent = 32

function Menu.file(x, y)
	local options = {
		indent = Ui.scale(indent),
		style = "menu",
		align = tessera.graphics.ALIGN_LEFT,
	}
	local items = {
		{
			widget = widgets.Button.new("New", options),
			action = file.new,
			tooltip = "ctrl+N",
		},
		{
			widget = widgets.Button.new("Open...", options),
			action = file.open,
			tooltip = "ctrl+O",
		},
		{
			widget = widgets.Button.new("Save", options),
			action = file.save,
			icon = tessera.icon.save,
			tooltip = "ctrl+S",
		},
		{
			widget = widgets.Button.new("Save as...", options),
			action = file.save_as,
			tooltip = "ctrl+shift+S",
		},
		{ type = "separator" },
		{
			widget = widgets.Button.new("Render audio", options),
			action = engine.render_start,
			tooltip = "ctrl+R",
		},
		{
			widget = widgets.Button.new("Open manual", options),
			action = function()
				local url =
					string.format("https://github.com/Sin-tel/tessera/blob/v%s/manual.md", util.version_str(VERSION))
				tessera.open_url(url)
			end,
		},
		{
			widget = widgets.Button.new("Quit", options),
			action = tessera.exit,
			tooltip = "escape",
		},
	}

	return Menu.new(items, x, y)
end

function Menu.options(x, y)
	local indent_s = Ui.scale(indent)
	local items = {
		{
			widget = widgets.Toggle.new(
				project.settings,
				"preview_notes",
				{ label = "Preview notes", style = "menu", pad = indent_s, size = 0.66, no_undo = true }
			),
		},
		{
			widget = widgets.Toggle.new(
				project.settings,
				"chase",
				{ label = "Chase notes", style = "menu", pad = indent_s, size = 0.66, no_undo = true }
			),
		},
		{
			widget = widgets.Toggle.new(
				project.settings,
				"follow",
				{ label = "Follow", style = "menu", pad = indent_s, size = 0.66, no_undo = true }
			),
		},
	}
	return Menu.new(items, x, y)
end

function Menu.new(items, x, y)
	local self = setmetatable({}, Menu)

	self.items = items

	self.x, self.y = x, y
	self.w = Ui.scale(260)
	self.h = 0 -- gets updated automatically

	if not x then
		self.x = mouse.x - 0.6 * self.w
		self.y = mouse.y - Ui.ROW_HEIGHT * 0.5
	end

	self.ui = Ui.new(self)
	self.ui.layout:padding(2)

	return self
end

local function draw_sep(x, y, w, h)
	tessera.graphics.set_color(theme.line)
	tessera.graphics.line(x + h, y + 0.5 * h, x + w - 2 * h, y + 0.5 * h)
end

local function draw_item(item, x, y, w, h)
	if item.icon then
		tessera.graphics.set_color(theme.ui_text)
		tessera.graphics.draw_path(item.icon, x + Ui.PAD, y)
	end
	if item.tooltip then
		tessera.graphics.set_color(theme.text_tip)
		tessera.graphics.label(item.tooltip, x - Ui.PAD, y, w, h, tessera.graphics.ALIGN_RIGHT)
	end
end

function Menu:update()
	self.ui:start_frame(self.x, self.y)

	for _, v in ipairs(self.items) do
		if v.type == "separator" then
			local x, y, w, h = self.ui:next(Ui.ROW_HEIGHT * 0.5)
			self.ui:push_draw(draw_sep, { x, y, w, h })
		else
			if v.widget:update(self.ui) then
				if v.action then
					v.action()
				end
				self.should_close = true
			end
			local x, y, w, h = self.ui.layout:get()
			self.ui:push_draw(draw_item, { v, x, y, w, h })
		end
	end

	self.ui:end_frame(self.x, self.y)

	self.h = self.ui.layout:total_height()
end

function Menu:get_mouse()
	return mouse.x, mouse.y
end

function Menu:focus()
	return util.hit(self, mouse.x, mouse.y)
end

function Menu:draw()
	tessera.graphics.set_color(theme.bg_menu)
	tessera.graphics.rectangle("fill", self.x, self.y, self.w, self.h, Ui.CORNER_RADIUS)

	self.ui:draw()
end

return Menu
