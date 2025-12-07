require("table.clear")
local View = require("view")
local midi = require("midi")

local Debug = View.derive("Debug")
Debug.__index = Debug

function Debug.new()
	local self = setmetatable({}, Debug)

	self.line_x = {}
	self.line_y = {}
	self.line_w = {}
	return self
end

function Debug:draw()
	local ix, iy = 32, 32
	local mx, my = self:get_mouse()

	tessera.graphics.set_color(theme.ui_text)

	-- local fsize = mx * 0.01 + 12
	local fsize = 18

	tessera.graphics.set_font_size(fsize)

	local ls = fsize * 1.5

	local w = self.w - ix
	tessera.graphics.set_font_notes()
	tessera.graphics.label("abcdefghijklmnopqrstu", ix, iy, w, ls)
	tessera.graphics.label("Ar Bs Ct Du Efea Fh Gi Bga", ix, iy + ls, w, ls)
	tessera.graphics.label('jA kB Cl Dm En Fo Gp Aq D"', ix, iy + 2 * ls, w, ls)
	tessera.graphics.label("abc - pnoq - jk - (a)", ix, iy + 3 * ls, w, ls)
	tessera.graphics.label("+-lm hci 5/4 7/8 11/8 - 4:5:6:7 Ac~Ba", ix, iy + 4 * ls, w, ls)

	tessera.graphics.set_font_main()
	tessera.graphics.label("abc - pnoq - jk - (a)", ix, iy + 5 * ls, w, ls)

	-- tessera.graphics.set_line_width(5)
	-- tessera.graphics.polyline(self.line_x, self.line_y)
	-- tessera.graphics.set_line_width()

	tessera.graphics.polyline_w(self.line_x, self.line_y, self.line_w)
end

function Debug:mousepressed()
	local mx, my = self:get_mouse()

	if mouse.button == 1 then
		table.insert(self.line_x, mx)
		table.insert(self.line_y, my)
		table.insert(self.line_w, math.random() * 6 + 1)
	else
		table.clear(self.line_x)
		table.clear(self.line_y)
		table.clear(self.line_w)
	end
end

return Debug
