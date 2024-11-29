local View = require("view")
local midi = require("midi")

local Debug = View:derive("Debug")

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

	love.graphics.setColor(theme.ui_text)

	-- util.drawText(dump(project), ix, iy, self.w, 0)
	local ls = 1.5 * resources.fonts.notes:getHeight()

	love.graphics.setFont(resources.fonts.notes)
	util.drawText("abcdefghijklmnopqrstu", ix, iy, self.w, 0)
	util.drawText("Ar Bs Ct Du Efea Fh Gi Bga", ix, iy + ls, self.w, 0)
	util.drawText('jA kB Cl Dm En Fo Gp Aq D"', ix, iy + 2 * ls, self.w, 0)
	util.drawText("abc - pnoq - jk - (a)", ix, iy + 3 * ls, self.w, 0)
	util.drawText("+-lm hci 5/4 7/8 11/8 - 4:5:6:7 Ac~Ba", ix, iy + 4 * ls, self.w, 0)

	-- TODO: remove
	-- local handler
	-- for i, ch in ipairs(build.list) do
	-- 	if ch.armed then
	-- 		handler = ch.midi_handler
	-- 		break
	-- 	end
	-- end

	-- if not handler then
	-- 	return
	-- end

	-- for i, v in ipairs(handler.voices) do
	-- 	util.drawText(tostring(v.note), ix, iy + 16 * i, w, 0)
	-- end
end

return Debug
