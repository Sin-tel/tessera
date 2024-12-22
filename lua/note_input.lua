local tuning = require("tuning")
local VoiceAlloc = require("voice_alloc")

local noteInput = {}

local key_down = {}

local DEFAULT_VELOCITY = 0.3

local octave = 0

noteInput.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function noteInput:keypressed(key)
	local ch_index = selection.ch_index
	if ch_index then
		for i, v in ipairs(self.diatonic_row) do
			if v == key then
				local p = tuning.fromDiatonic(i, octave)
				local id = VoiceAlloc.next_id()
				key_down[i] = id
				ui_channels[ch_index].roll:event({ name = "note_on", id = id, pitch = p, vel = DEFAULT_VELOCITY })
				return true
			end
		end
	end

	return false
end

function noteInput:keyreleased(key)
	local ch_index = selection.ch_index
	if ch_index then
		for i, v in ipairs(self.diatonic_row) do
			if v == key then
				local id = key_down[i]
				if id then
					ui_channels[ch_index].roll:event({ name = "note_off", id = id })
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
	end
	return false
end

return noteInput
