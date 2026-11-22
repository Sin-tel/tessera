-- Base for tools that reshape notes near the mouse.
-- `local b = brush.new(name, radius)`, then define `b:stroke(note, weights)`. It's called every frame for
-- each note in reach, with a falloff in 0..1 per vert (1 under the mouse). Each brush scales it by its own strength.
-- Works on the selection, or else on the note closest to where the stroke started.

local Ui = require("ui.ui")
local curve = require("curve")
local tuning = require("tuning")

local brush = {}
brush.__index = brush

local MIN_WEIGHT = 0.002
-- in radii, where the weight drops below MIN_WEIGHT
local REACH = 2.5

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

-- Snapshot for undo, and give the brush evenly spaced verts to work on. Simplified again on release.
function brush:touch(note)
	self.touched[note] = true
	table.insert(self.prev_state, util.clone(note))
	table.insert(self.notes, note)

	note.verts = curve.subdivide(note.verts)
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
		local w = math.exp(-(0.69 * dx * dx + dy * dy) / r_sq)
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
			self:touch(note)
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
	for _, note in ipairs(self.notes) do
		note.verts = curve.simplify(note.verts)
	end
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
