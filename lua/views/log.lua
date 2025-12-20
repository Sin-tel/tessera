local Ui = require("ui.ui")
local View = require("view")
local log = require("log")

local Log = View.derive("Log")
Log.__index = Log

function Log.new()
	local self = setmetatable({}, Log)

	return self
end

function Log:draw()
	local pad = Ui.scale(32)
	local x = pad
	local y = pad
	local w = self.w - 2 * pad
	local h = self.h - 2 * pad

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.set_font_size(16)
	tessera.graphics.set_font_main()

	local text = table.concat(log.messages, "\n")

	tessera.graphics.text_wrapped(text, x, y, w, h)
end

return Log
