local Pitch = require("pitch")
local backend = require("backend")

local noteInput = {}

local queue = {}

local DEFAULT_VELOCITY = 0.75

local octave = 0

noteInput.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function noteInput:keypressed(key, _)
	local handled = false
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local ch = selection.channel

			local p = Pitch:fromDiatonic(i, octave)

			if ch then
				backend:sendNote(ch.index, p.pitch, DEFAULT_VELOCITY)
			end

			table.insert(queue, i)

			handled = true
			break
		end
	end

	return handled
end

function noteInput:keyreleased(key)
	local handled = false
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local last = false
			for j, b in ipairs(queue) do
				if b == i then
					if j == #queue then
						last = true
					end
					table.remove(queue, j)
					break
				end
			end
			local ch = selection.channel

			if ch then
				if #queue > 0 then
					if last then
						local p = Pitch:fromDiatonic(queue[#queue], octave)
						backend:sendNote(ch.index, p.pitch, DEFAULT_VELOCITY)
					end
				else
					local p = Pitch:fromDiatonic(i, octave)
					backend:sendNote(ch.index, p.pitch, 0.0)
				end
			end

			handled = true
		end
	end

	if key == "z" then
		octave = octave - 1
		if octave < -4 then
			octave = -4
		end
		handled = true
	elseif key == "x" then
		octave = octave + 1
		if octave > 4 then
			octave = 4
		end
		handled = true
	end

	return handled
end

return noteInput
