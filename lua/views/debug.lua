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
	local ix, iy = 20, 20

	love.graphics.setColor(theme.ui_text)

	-- util.drawText(dump(project), ix, iy, self.w, 0)

	love.graphics.setFont(resources.fonts.notes)
	util.drawText("abcdefghijklmnopqrstu (c) ABCDEFG", ix, iy, self.w, 0)
	-- util.drawText("B5a Cd Eg Afea Gh Ei Bga", ix, iy + 25, self.w, 0)
	util.drawText("Bk Br Bs Ct Cu Afea Gh Ei Bga", ix, iy + 25, self.w, 0)
	util.drawText("Aj Bk Cl Dm En Fo Gp Aq", ix, iy + 50, self.w, 0)
	util.drawText("-abc-pnoq-jk-Af A!", ix, iy + 75, self.w, 0)
	util.drawText("+-lm hci 5/4 7/8 11/8 - 4:5:6:7", ix, iy + 100, self.w, 0)
	util.drawText("(c) (a)", ix, iy + 125, self.w, 0)
	-- love.graphics.setFont(resources.fonts.smufl)

	-- util.drawText("\xEE\x89\xA0-\xEE\x89\xA1-\xEE\x89\xA2-\xEE\x89\xA3-\xEE\x89\xA4-\xEE\x89\xA6", ix, iy, self.w, 0)
	-- util.drawText("\xEE\x89\xA0-\xEE\x89\xBA-\xEE\x89\xBB-\xEE\x8B\xA2", ix, iy + 24, self.w, 0)

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
