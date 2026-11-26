-- Moves everything right of the press along with the mouse, falling off to the left. Inserts or removes time,
-- or bends the pitch of what follows. Shift: time only, ctrl: pitch only.
-- Pitch moves the offsets, notes keep their interval.
-- Works on the selection, or else the whole active channel.

local Ui = require("ui.ui")
local brush = require("tools/brush")
local curve = require("curve")

local stretch = {}
stretch.name = "stretch"
-- pixels left of the press where half the movement is left
stretch.radius = Ui.scale(200)

-- weights below this are left alone
local MIN_WEIGHT = 0.002
-- Leftwards drag limit, in radii. The falloff's steepest slope is ~0.71 / radius, so verts stay in order
-- while dragging less than ~1.4 radii to the left.
local MAX_LEFT = 1.35

-- weight of a point `x` pixels right of the press
function stretch:weight(x)
	if x >= 0 then
		return 1
	end
	return math.exp(-0.69 * (x / self.radius) ^ 2)
end

function stretch:mousepressed(canvas)
	self.x0, self.y0 = canvas:get_mouse()
	self.dx = 0
	self.moved = false
	self.notes = {}
	self.prev_state = {}
	-- per note: time and verts at press, weight per vert
	self.orig = {}
	-- notes in the falloff, resampled so they bend smoothly
	self.resampled = {}

	local scope
	if not selection.is_empty() then
		scope = selection.list
	else
		local channel = selection.active_channel()
		scope = channel and channel.notes or {}
	end

	local transform = canvas.transform
	for _, note in ipairs(scope) do
		-- weight only grows to the right, so the note's end has the most
		local x_end = transform:time(note.time + note.verts[#note.verts][1]) - self.x0
		if self:weight(x_end) > MIN_WEIGHT then
			table.insert(self.prev_state, util.clone(note))
			table.insert(self.notes, note)
			if transform:time(note.time) < self.x0 then
				note.verts = curve.subdivide(note.verts)
				self.resampled[note] = true
			end
			local weights = {}
			for i, v in ipairs(note.verts) do
				weights[i] = self:weight(transform:time(note.time + v[1]) - self.x0)
			end
			self.orig[note] = { time = note.time, verts = util.clone(note.verts), weights = weights }
		end
	end
end

function stretch:mousedown(canvas)
	local mx, my = canvas:get_mouse()
	local dx = math.max(mx - self.x0, -MAX_LEFT * self.radius)
	local dy = my - self.y0
	if modifier_keys.shift then
		dy = 0
	elseif modifier_keys.ctrl then
		dx = 0
	end
	self.dx = dx
	self.moved = self.moved or dx ~= 0 or dy ~= 0

	local dt = dx / canvas.transform.sx
	local dp = dy / canvas.transform.sy
	for note, o in pairs(self.orig) do
		local verts = note.verts
		for i, v in ipairs(o.verts) do
			local w = o.weights[i]
			verts[i][1] = v[1] + dt * w
			verts[i][2] = v[2] + dp * w
		end
		note.time = o.time
		brush.fix_start(note)
	end
end

function stretch:mousereleased()
	local notes, prev_state = self.notes, self.prev_state
	self.notes = nil
	self.orig = nil

	if not self.moved then
		-- a click alone leaves the notes as they were
		for i, note in ipairs(notes) do
			note.verts = prev_state[i].verts
		end
		return
	end
	for note in pairs(self.resampled) do
		note.verts = curve.simplify(note.verts)
	end
	if #notes > 0 then
		command.register(command.NoteUpdate.new(prev_state, notes))
	end
end

-- A line where the press is (or would be), and a fainter one where half the movement is left.
function stretch:draw(canvas)
	local mx = canvas:get_mouse()
	local x = self.notes and self.x0 + self.dx or mx
	local c = theme.text_tip
	tessera.graphics.set_line_width(1.5)
	tessera.graphics.set_color(c)
	tessera.graphics.line(x, 0, x, canvas.h)
	tessera.graphics.set_color_f(c[1], c[2], c[3], 0.4)
	tessera.graphics.line(x - self.radius, 0, x - self.radius, canvas.h)
end

return stretch
