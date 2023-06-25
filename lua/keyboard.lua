-- TODO: just make this emulate a midi input to avoid code duplication
-- TODO: rename this module, keyboard is confusing

local Pitch = require("pitch")
local backend = require("backend")

local keyboard = {}

keyboard.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function keyboard:keypressed(key, _)
	local handled = false
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local ch = selection.channel

			local p = Pitch:newFromDiatonic(i)

			if ch then
				backend:send_note_on(ch.index, p.pitch, 0.5)
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
				backend:send_cv(ch.index, p.pitch, 0.0)
			end

			handled = true
		end
	end

	return handled
end

return keyboard
