local View = require("view")
local midi = require("midi")

local Debug = View:derive("Debug")

function Debug:draw()
	local ix, iy = 20, 20
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	love.graphics.setColor(theme.ui_text)

	-- TODO: remove
	local handler
	for i, ch in ipairs(channelHandler.list) do
		if ch.armed then
			handler = ch.midi_handler
			break
		end
	end

	if not handler then
		return
	end

	for i, v in ipairs(handler.voices) do
		util.drawText(tostring(v.note), ix, iy + 16 * i, w, 0)
	end
end

return Debug
