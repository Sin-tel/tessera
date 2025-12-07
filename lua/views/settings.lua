local View = require("view")
local Settings = View.derive("Settings")
Settings.__index = Settings

function Settings.new()
	local self = setmetatable({}, Settings)

	return self
end

function Settings:draw()
	local ix, iy = 32, 32
	local mx, my = self:get_mouse()

	tessera.graphics.set_color(theme.ui_text)

	tessera.graphics.set_font_main()
	tessera.graphics.set_font_size()

	tessera.graphics.text("This is where the settings go.", ix, iy)
end

return Settings
