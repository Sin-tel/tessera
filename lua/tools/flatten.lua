local brush = require("tools/brush")

local flatten = brush.new("flatten", 30)

-- pull per frame under the mouse
local STRENGTH = 0.25

function flatten:mousepressed(canvas)
	brush.mousepressed(self, canvas)
	-- per note, pitch to flatten to
	self.levels = {}
end

-- Pulls pitch towards a level per note, sampled where the stroke first touches it (at the press, for the note
-- under it). The level is a narrower-weighted average (weights ^ 4, half the radius) of the pitch there.
function flatten:stroke(note, weights)
	local verts = note.verts

	local level = self.levels[note]
	if not level then
		local sum, avg = 0, 0
		for i, v in ipairs(verts) do
			local w = weights[i] ^ 4
			avg = avg + v[2] * w
			sum = sum + w
		end
		if sum == 0 then
			return
		end
		level = avg / sum
		self.levels[note] = level
	end

	for i, v in ipairs(verts) do
		v[2] = v[2] + (level - v[2]) * weights[i] * STRENGTH
	end
end

return flatten
