local util = require("util")

local scale = {}

-- TODO: scale pitch offsets

function scale:mousepressed(canvas)
	self.ix, self.iy = mouse.x, mouse.y
	self.start_x, self.start_y = canvas.transform:inverse(mouse.x, mouse.y)
	self.prev_state = util.clone(selection.list)
	self.edit = false

	self.start_time = math.huge
	self.end_time = -math.huge
	for i, v in ipairs(selection.list) do
		if v.time < self.start_time then
			self.start_time = v.time
		end

		local n = #v.verts
		local t_end = v.time + v.verts[n][1]

		if t_end > self.end_time then
			self.end_time = t_end
		end
	end
end

function scale:mousedown(canvas)
	if not self.edit and util.dist(self.ix, self.iy, mouse.x, mouse.y) < mouse.DRAG_DIST then
		return
	else
		self.edit = true
	end

	local mx, _ = canvas.transform:inverse(mouse.x, mouse.y)

	local t_total = self.end_time - self.start_time

	local s = 1.0 + (mx - self.start_x) / t_total

	s = math.max(0.02, s)

	for i, v in ipairs(selection.list) do
		local t_rel = self.prev_state[i].time - self.start_time
		v.time = self.start_time + t_rel * s

		for j in ipairs(v.verts) do
			v.verts[j][1] = s * self.prev_state[i].verts[j][1]
		end
	end
end

function scale:mousereleased(canvas)
	if self.edit then
		local c = command.NoteUpdate.new(self.prev_state, selection.list)
		command.register(c)
		self.prev_state = nil
	end
end

function scale:draw(canvas) end

return scale
