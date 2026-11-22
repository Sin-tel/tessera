local Ui = require("ui.ui")
local curve = require("curve")
local note_input = require("note_input")
local smooth = require("tools/smooth")
local tuning = require("tuning")

-- Draw a note freehand. The line trails the mouse by `radius`, which steadies it.
-- Drawing back over the line replaces it, so it stays a function of time.

local freehand = {}
freehand.modifiers = { shift = smooth }

freehand.name = "freehand"
freehand.radius = Ui.scale(5)

-- Adds the pen position as a point { time, pitch, pressure }, removing what it was drawn back over.
function freehand:add_point(canvas)
	local t = canvas.transform:time_inv(self.x)
	local p = canvas.transform:pitch_inv(self.y)

	if self.last then
		local t1, t2 = math.min(self.last[1], t), math.max(self.last[1], t)
		for i = #self.points, 1, -1 do
			local pt = self.points[i]
			if (pt[1] > t1 and pt[1] < t2) or pt[1] == t then
				table.remove(self.points, i)
			end
		end
	end

	-- no tablet yet
	local pt = { t, p, note_input.DEFAULT_PRESSURE }
	local i = #self.points + 1
	while i > 1 and self.points[i - 1][1] > t do
		i = i - 1
	end
	table.insert(self.points, i, pt)
	self.last = pt
end

function freehand:mousepressed(canvas)
	self.points = nil
	self.last = nil

	local ch_index = selection.ch_index
	if not ch_index then
		return
	end
	local channel = project.channels[ch_index]
	if channel.lock or not channel.visible or not channel.notes then
		return
	end

	self.ch_index = ch_index
	self.x, self.y = canvas:get_mouse()
	self.points = {}
	self:add_point(canvas)

	if project.settings.preview_notes then
		local pitch = self.last[2]
		self.interval = tuning.snap(pitch)
		self.token = tessera.audio.get_token()
		ui_channels[ch_index]:send_event({
			name = "note_on",
			token = self.token,
			interval = self.interval,
			vel = note_input.DEFAULT_VELOCITY,
			offset = pitch - tuning.get_pitch(self.interval),
		})
	end
end

function freehand:mousedown(canvas)
	if not self.points then
		return
	end

	local mx, my = canvas:get_mouse()
	local dx, dy = mx - self.x, my - self.y
	local d = math.sqrt(dx * dx + dy * dy)
	if d <= self.radius then
		return
	end
	self.x = mx - self.radius * dx / d
	self.y = my - self.radius * dy / d
	self:add_point(canvas)

	if self.token then
		ui_channels[self.ch_index]:send_event({
			name = "pitch",
			token = self.token,
			offset = self.last[2] - tuning.get_pitch(self.interval),
		})
	end
end

function freehand:mousereleased(canvas)
	local points = self.points
	self.points = nil
	self.last = nil

	if self.token then
		ui_channels[self.ch_index]:send_event({ name = "note_off", token = self.token })
		self.token = nil
	end

	if not points or #points < 2 then
		return
	end

	local t0 = points[1][1]
	local interval = tuning.snap(points[1][2])
	local base = tuning.get_pitch(interval)

	local verts = {}
	for i, pt in ipairs(points) do
		verts[i] = { pt[1] - t0, pt[2] - base, pt[3] }
	end
	verts = curve.simplify(verts)

	local note = { interval = interval, time = t0, vel = note_input.DEFAULT_VELOCITY, verts = verts }
	command.run_and_register(command.NoteAdd.new({ [self.ch_index] = { note } }))
	return true
end

function freehand:draw(canvas)
	local mx, my = canvas:get_mouse()
	if not (self.points or canvas:focus()) then
		return
	end

	tessera.graphics.set_color(theme.text_tip)
	tessera.graphics.circle("line", mx, my, self.radius)

	if not self.points or #self.points < 2 then
		return
	end

	local ch = project.channels[self.ch_index]
	tessera.graphics.set_color(tessera.graphics.get_color_hsv(ch.hue / 360, 0.75, 0.80))
	local lx, ly = {}, {}
	for i, pt in ipairs(self.points) do
		lx[i] = canvas.transform:time(pt[1])
		ly[i] = canvas.transform:pitch(pt[2])
	end
	tessera.graphics.set_line_width(2.0)
	tessera.graphics.polyline(lx, ly)
	tessera.graphics.line(self.x, self.y, mx, my)
end

return freehand
