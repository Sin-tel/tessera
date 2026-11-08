-- Base for tools that reshape notes near the mouse.
-- `local b = brush.new(name, radius)`, then define `b:stroke(note, weights)`. It's called every frame for
-- each note in reach, with a weight in 0..1 per vert.
-- Works on the selection, or else on the note closest to where the stroke started.

local Ui = require("ui.ui")
local curve = require("curve")
local tuning = require("tuning")

local brush = {}
brush.__index = brush

-- weight in the middle of the brush
local STRENGTH = 0.5
local MIN_WEIGHT = 0.001
-- in radii, where the weight drops below MIN_WEIGHT
local REACH = 2.5
-- vert spacing on touched notes, in radii
local SPACING = 0.8
-- seconds, about as dense as recorded notes
local MIN_SPACING = 0.008

function brush.new(name, radius)
	return setmetatable({ name = name, radius = Ui.scale(radius) }, brush)
end

function brush:mousepressed(canvas)
	self.touched = {}
	self.notes = {}
	self.prev_state = {}

	if selection.is_empty() then
		local mx, my = canvas:get_mouse()
		local closest = canvas:find_closest_note(mx, my, math.huge)
		self.scope = { closest }
	else
		self.scope = selection.list
	end
end

-- Snapshot for undo, and give the brush evenly spaced verts to work on.
function brush:touch(canvas, note)
	self.touched[note] = true
	table.insert(self.prev_state, util.clone(note))
	table.insert(self.notes, note)

	local max_dt = math.max(SPACING * self.radius / canvas.transform.sx, MIN_SPACING)
	note.verts = curve.subdivide(curve.simplify(note.verts), max_dt)
end

function brush:get_weights(canvas, note, mx, my)
	local transform = canvas.transform
	local base_pitch = tuning.get_pitch(note.interval)
	local r_sq = self.radius ^ 2

	local weights = {}
	local any = false
	for i, v in ipairs(note.verts) do
		local dx = transform:time(note.time + v[1]) - mx
		local dy = transform:pitch(base_pitch + v[2]) - my
		local w = STRENGTH * math.exp(-(0.69 * dx * dx + dy * dy) / r_sq)
		weights[i] = w
		any = any or w > MIN_WEIGHT
	end
	return any and weights
end

function brush:mousedown(canvas)
	local mx, my = canvas:get_mouse()
	local reach = REACH * self.radius

	for _, note in ipairs(self.scope) do
		if not self.touched[note] and canvas:dist_sq_note(note, mx, my) < reach * reach then
			self:touch(canvas, note)
		end
		if self.touched[note] then
			local weights = self:get_weights(canvas, note, mx, my)
			if weights then
				self:stroke(note, weights)
			end
		end
	end
end

function brush:mousereleased()
	self.scope = nil
	if #self.notes > 0 then
		command.register(command.NoteUpdate.new(self.prev_state, self.notes))
	end
end

function brush:draw(canvas)
	if not (self.scope or canvas:focus()) then
		return
	end
	local mx, my = canvas:get_mouse()
	tessera.graphics.set_color(theme.text_tip)
	tessera.graphics.circle("line", mx, my, self.radius)
end

return brush
