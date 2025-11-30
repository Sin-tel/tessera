local View = require("view")
local midi = require("midi")

local Debug = View.derive("Debug")
Debug.__index = Debug

function Debug.new()
	local self = setmetatable({}, Debug)
	return self
end

function Debug:draw()
	local ix, iy = 32, 32

	tessera.graphics.set_color(theme.ui_text)

	-- local ls = 32

	-- tessera.graphics.set_font_notes()
	-- tessera.graphics.label("abcdefghijklmnopqrstu", ix, iy, self.w, 0)
	-- tessera.graphics.label("Ar Bs Ct Du Efea Fh Gi Bga", ix, iy + ls, self.w, 0)
	-- tessera.graphics.label('jA kB Cl Dm En Fo Gp Aq D"', ix, iy + 2 * ls, self.w, 0)
	-- tessera.graphics.label("abc - pnoq - jk - (a)", ix, iy + 3 * ls, self.w, 0)
	-- tessera.graphics.label("+-lm hci 5/4 7/8 11/8 - 4:5:6:7 Ac~Ba", ix, iy + 4 * ls, self.w, 0)

	tessera.graphics.set_line_width(5)
	tessera.graphics.polyline({ 24, 24, 60, 60, 90, 100 })
	tessera.graphics.set_line_width()
end

return Debug
