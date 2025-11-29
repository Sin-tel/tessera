local View = require("view")
local midi = require("midi")

local Debug = View.derive("Debug")
Debug.__index = Debug

local function dump(t, indent)
	indent = indent or 0
	if type(t) == "table" then
		local res = ""
		for k, v in pairs(t) do
			if type(v) == "table" then
				res = res .. string.rep("  ", indent) .. tostring(k) .. ":\n"
				res = res .. dump(v, indent + 1)
			else
				local s = tostring(v)
				if type(v) == "string" then
					s = '"' .. s .. '"'
				end
				res = res .. string.rep("  ", indent) .. tostring(k) .. ": " .. s .. "\n"
			end
		end
		return res
	else
		return tostring(t) .. "\n"
	end
end

function Debug:draw()
	local ix, iy = 32, 32

	tessera.graphics.set_color(theme.ui_text)

	-- util.draw_text(dump(project), ix, iy, self.w, 0)
	local ls = 1.5 * resources.fonts.notes:get_height()

	tessera.graphics.set_font(resources.fonts.notes)
	util.draw_text("abcdefghijklmnopqrstu", ix, iy, self.w, 0)
	util.draw_text("Ar Bs Ct Du Efea Fh Gi Bga", ix, iy + ls, self.w, 0)
	util.draw_text('jA kB Cl Dm En Fo Gp Aq D"', ix, iy + 2 * ls, self.w, 0)
	util.draw_text("abc - pnoq - jk - (a)", ix, iy + 3 * ls, self.w, 0)
	util.draw_text("+-lm hci 5/4 7/8 11/8 - 4:5:6:7 Ac~Ba", ix, iy + 4 * ls, self.w, 0)
end

return Debug
