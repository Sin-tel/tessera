local tuning = require("tuning")

local note_input = {}

local key_down = {}

local DEFAULT_VELOCITY = 0.5

local octave = 0

note_input.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function note_input:keypressed(key)
	local ch_index = selection.ch_index
	if ch_index then
		for i, v in ipairs(self.diatonic_row) do
			if v == key then
				local p = tuning.from_diatonic(i, octave)
				local token = tessera.audio.get_token()
				key_down[i] = token
				ui_channels[ch_index]:event({ name = "note_on", token = token, pitch = p, vel = DEFAULT_VELOCITY })
				return true
			end
		end
	end

	return false
end

function note_input:keyreleased(key)
	local ch_index = selection.ch_index
	if ch_index then
		for i, v in ipairs(self.diatonic_row) do
			if v == key then
				local token = key_down[i]
				if token then
					ui_channels[ch_index]:event({ name = "note_off", token = token })
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

return note_input
