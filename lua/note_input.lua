local tuning = require("tuning")

local note_input = {}

-- held keys, with the channel their note went to
local key_down = {}

-- velocity for notes entered without one (computer keyboard, pen tool)
note_input.DEFAULT_VELOCITY = 0.7

local octave = 0

note_input.diatonic_row = { "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]" }

function note_input:keypressed(key)
	local ch_index = selection.ch_index
	if ch_index then
		for i, v in ipairs(self.diatonic_row) do
			if v == key then
				local o = tuning.mul(tuning.octave, octave)
				local interval = tuning.from_table(tuning.diatonic, i - 1)
				interval = tuning.add(interval, o)
				local token = tessera.audio.get_token()
				local channel = ui_channels[ch_index]
				key_down[i] = { token = token, channel = channel }
				channel:event({
					name = "note_on",
					token = token,
					interval = interval,
					vel = note_input.DEFAULT_VELOCITY,
					offset = 0,
				})
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

-- The note off goes to the channel the note started on, even if the selection changed since.
function note_input:keyreleased(key)
	for i, v in ipairs(self.diatonic_row) do
		if v == key then
			local held = key_down[i]
			if held then
				held.channel:event({ name = "note_off", token = held.token })
				key_down[i] = nil
				return true
			end
			return selection.ch_index ~= nil
		end
	end
	return false
end

return note_input
