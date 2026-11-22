-- Drags the curve near where the drag started. Falls off with time along the note, so the shape
-- (e.g. vibrato) moves as a whole, and with the note's distance from the press.
-- Dragging a note's start moves the note.

local brush = require("tools/brush")
local smooth = require("tools/smooth")

local grab = brush.new("grab", 40)
grab.modifiers = { shift = smooth }

-- Time falloff radius per pixel dragged sideways, once that's past the brush radius.
-- Above ~0.72 verts can't cross; higher spreads the move wider and bunches verts less.
local TIME_FALLOFF = 1.2

function grab:mousepressed(canvas)
	brush.mousepressed(self, canvas)
	local mx, my = canvas:get_mouse()
	self.x0, self.y0 = mx, my
	self.moved = false
	-- per touched note: time and verts at press, weight, vert x offsets from the press
	self.orig = {}

	local transform = canvas.transform
	local reach = brush.REACH * self.radius
	for _, note in ipairs(self.scope) do
		local d_sq = canvas:dist_sq_note(note, mx, my)
		if d_sq < reach * reach then
			self:touch(note)
			local offsets = {}
			for i, v in ipairs(note.verts) do
				offsets[i] = transform:time(note.time + v[1]) - mx
			end
			local weight = self:note_weight(d_sq)
			self.orig[note] = { time = note.time, verts = util.clone(note.verts), weight = weight, offsets = offsets }
		end
	end
end

function grab:mousedown(canvas)
	local mx, my = canvas:get_mouse()
	local dx, dy = mx - self.x0, my - self.y0
	self.moved = self.moved or dx ~= 0 or dy ~= 0

	local r_sq = self.radius ^ 2
	-- Time falloff widens the further it's dragged, so verts don't cross.
	local rx_sq = math.max(self.radius, TIME_FALLOFF * math.abs(dx)) ^ 2
	local dt = dx / canvas.transform.sx
	local dp = dy / canvas.transform.sy

	for note, o in pairs(self.orig) do
		local verts = note.verts
		for i, v in ipairs(o.verts) do
			local ox_sq = o.offsets[i] ^ 2
			verts[i][1] = v[1] + dt * o.weight * math.exp(-0.69 * ox_sq / rx_sq)
			verts[i][2] = v[2] + dp * o.weight * math.exp(-0.69 * ox_sq / r_sq)
		end

		note.time = o.time
		brush.fix_start(note)
	end
end

function grab:mousereleased(canvas)
	-- a click alone leaves the notes as they were
	if not self.moved then
		for i, note in ipairs(self.notes) do
			note.verts = self.prev_state[i].verts
		end
		self.notes = {}
	end
	self.orig = nil
	brush.mousereleased(self, canvas)
end

return grab
