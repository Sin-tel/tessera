local tuning = require("tuning")
local backend = require("backend")

local noteInput = {}

local queue = {}

local DEFAULT_VELOCITY = 0.3

local octave = 0

-- TODO: this should not communicate with backend directly
--       instead, pass through the selected devices' input handler

noteInput.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function noteInput:keypressed(key)
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local p = tuning.getPitch(tuning.fromDiatonic(i, octave))

			local ch_index = selection.channel_index
			if ch_index then
				backend:noteOn(ch_index, p, DEFAULT_VELOCITY)
			end

			table.insert(queue, i)

			return true
		end
	end

	return false
end

function noteInput:keyreleased(key)
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
			local ch_index = selection.channel_index

			if ch_index then
				if #queue > 0 then
					if last then
						local p = tuning.getPitch(tuning.fromDiatonic(queue[#queue], octave))
						backend:noteOn(ch_index, p, DEFAULT_VELOCITY)
					end
				else
					backend:noteOff(ch_index)
				end
			end

			return true
		end
	end

	if key == "z" then
		octave = octave - 1
		if octave < -4 then
			octave = -4
		end
		return true
	elseif key == "x" then
		octave = octave + 1
		if octave > 4 then
			octave = 4
		end
		return true
	end

	return false
end

return noteInput
