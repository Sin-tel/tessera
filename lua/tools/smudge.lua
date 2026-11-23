-- Pushes the curve along with the mouse, every frame by how far it moved, so it can be stroked repeatedly.
-- Moving a note's start moves the note.

local brush = require("tools/brush")
local smooth = require("tools/smooth")

local smudge = brush.new("smudge", 20, "time")
smudge.modifiers = { shift = smooth }
smudge.whole_channel = true

-- share of the mouse movement under the mouse
local STRENGTH = 0.4
-- per frame, in radii. Below ~1.4 / STRENGTH verts can't cross.
local MAX_STEP = 1

function smudge:mousepressed(canvas)
	brush.mousepressed(self, canvas)
	self.px, self.py = canvas:get_mouse()
end

function smudge:mousedown(canvas)
	local mx, my = canvas:get_mouse()
	local max = MAX_STEP * self.radius
	self.dx = math.min(math.max(mx - self.px, -max), max)
	self.dy = my - self.py
	self.px, self.py = mx, my
	if self.dx ~= 0 or self.dy ~= 0 then
		brush.mousedown(self, canvas)
	end
end

function smudge:stroke(note, weights, canvas)
	local dt = self.dx * STRENGTH / canvas.transform.sx
	local dp = self.dy * STRENGTH / canvas.transform.sy
	for i, v in ipairs(note.verts) do
		v[1] = v[1] + dt * weights[i]
		v[2] = v[2] + dp * weights[i]
	end
	brush.fix_start(note)
end

return smudge
