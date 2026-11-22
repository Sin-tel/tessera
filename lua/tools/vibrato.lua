-- Paints vibrato onto the pitch, building up the longer it's brushed. Ctrl: adjust pressure (tremolo).
-- Phase starts at the note start.

local brush = require("tools/brush")
local smooth = require("tools/smooth")

local vibrato = brush.new("vibrato", 40, "time")
vibrato.modifiers = { shift = smooth }

-- Hz
local RATE = 5.5
-- semitones added per frame under the mouse
local DEPTH = 0.01

function vibrato:mousepressed(canvas)
	brush.mousepressed(self, canvas)
end

function vibrato:stroke(note, weights)
	local k = 2 * math.pi * RATE
	if modifier_keys.ctrl then
		for i, v in ipairs(note.verts) do
			local w = v[3] + DEPTH * weights[i] * math.cos(k * v[1])
			w = util.clamp(w, 0.0, 1.0)
			v[3] = w
		end
	else
		for i, v in ipairs(note.verts) do
			v[2] = v[2] + DEPTH * weights[i] * math.sin(k * v[1])
		end
	end
end

return vibrato
