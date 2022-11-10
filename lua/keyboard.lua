local Pitch = require("pitch")
local audiolib = require("audiolib")

local keyboard = {}

keyboard.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function keyboard:keypressed(key, isrepeat)
	local handled = false
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local ch = selection.channel

			-- p = Pitch:new()
			local p = Pitch:newFromDiatonic(i)

			if ch then
				audiolib.send_noteOn(ch.index, p.pitch, 0.5)
			end

			handled = true
			break
		end
	end

	return handled
end

function keyboard:keyreleased(key)
	local handled = false
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local ch = selection.channel

			local p = Pitch:newFromDiatonic(i)

			if ch then
				audiolib.send_CV(ch.index, p.pitch, 0.0)
			end

			handled = true
		end
	end

	return handled
end

return keyboard
