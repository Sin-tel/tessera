-- Base for tools that reshape notes near the mouse.
-- `local b = brush.new(name, radius, falloff)`, then define `b:stroke(note, weights, canvas)`. It's called every
-- frame for each note in reach, with a falloff in 0..1 per vert (1 under the mouse). Each brush scales it by its
-- own strength. Works on the selection, or else on the note closest to where the stroke started
-- (`b.whole_channel = true`: on every note of the active channel instead).
--
-- falloff "2d" (default): by distance on screen, so the brush reaches what's under it.
-- falloff "time": by time along the note, times a weight for the note's distance from the mouse. Moves a note's
-- shape (e.g. vibrato) as a whole: with "2d", verts level with the mouse would get more than the peaks.

local Ui = require("ui.ui")
local curve = require("curve")
local tuning = require("tuning")

local brush = {}
brush.__index = brush

local MIN_WEIGHT = 0.002
-- in radii, where the weight drops below MIN_WEIGHT
brush.REACH = 2.5
-- seconds, closest verts get after moving them in time (rounding only, the falloffs keep them apart)
local MIN_DT = 0.001

function brush.new(name, radius, falloff)
	return setmetatable({ name = name, radius = Ui.scale(radius), falloff = falloff or "2d" }, brush)
end

-- After moving verts in time: the first vert goes back to 0 and the note moves instead.
function brush.fix_start(note)
	local verts = note.verts
	local t0 = verts[1][1]
	note.time = note.time + t0
	verts[1][1] = 0
	for i = 2, #verts do
		verts[i][1] = math.max(verts[i][1] - t0, verts[i - 1][1] + MIN_DT)
	end
end

function brush:mousepressed(canvas)
	self.touched = {}
	self.notes = {}
	self.prev_state = {}

	if not selection.is_empty() then
		self.scope = selection.list
	elseif self.whole_channel then
		local channel = selection.active_channel()
		self.scope = channel and channel.notes or {}
	else
		local mx, my = canvas:get_mouse()
		local closest = canvas:find_closest_note(mx, my, math.huge)
		self.scope = { closest }
	end
end

-- Snapshot for undo, and give the brush evenly spaced verts to work on. Simplified again on release.
function brush:touch(note)
	self.touched[note] = true
	table.insert(self.prev_state, util.clone(note))
	table.insert(self.notes, note)

	note.verts = curve.subdivide(note.verts)
end

-- Weight of a whole note, by its squared distance from the mouse.
function brush:note_weight(d_sq)
	return math.exp(-d_sq / self.radius ^ 2)
end

function brush:get_weights(canvas, note, mx, my)
	local transform = canvas.transform
	local base_pitch = tuning.get_pitch(note.interval)
	local r_sq = self.radius ^ 2
	local note_w = self.falloff == "time" and self:note_weight(canvas:dist_sq_note(note, mx, my))

	local weights = {}
	local any = false
	for i, v in ipairs(note.verts) do
		local dx = transform:time(note.time + v[1]) - mx
		local w
		if note_w then
			w = note_w * math.exp(-0.69 * dx * dx / r_sq)
		else
			local dy = transform:pitch(base_pitch + v[2]) - my
			w = math.exp(-(0.69 * dx * dx + dy * dy) / r_sq)
		end
		weights[i] = w
		any = any or w > MIN_WEIGHT
	end
	return any and weights
end

function brush:mousedown(canvas)
	local mx, my = canvas:get_mouse()
	local reach = brush.REACH * self.radius
	local transform = canvas.transform

	for _, note in ipairs(self.scope) do
		if not self.touched[note] then
			-- cheap check on the time range first, the scope can be a whole channel
			local x0 = transform:time(note.time)
			local x1 = transform:time(note.time + note.verts[#note.verts][1])
			if x0 < mx + reach and x1 > mx - reach and canvas:dist_sq_note(note, mx, my) < reach * reach then
				self:touch(note)
			end
		end
		if self.touched[note] then
			local weights = self:get_weights(canvas, note, mx, my)
			if weights then
				self:stroke(note, weights, canvas)
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
