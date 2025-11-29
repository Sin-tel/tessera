local util = require("util")

local drag_end = {}

local t_min = (1 / 16)

function drag_end:mousepressed(canvas)
	self.ix, self.iy = mouse.x, mouse.y
	self.start_x, self.start_y = canvas.transform:inverse(mouse.x, mouse.y)
	self.prev_state = util.clone(selection.list)
	self.edit = false
end

function drag_end:mousedown(canvas)
	if not self.edit and util.dist(self.ix, self.iy, mouse.x, mouse.y) < mouse.DRAG_DIST then
		return
	else
		self.edit = true
	end

	local mx, _ = canvas.transform:inverse(mouse.x, mouse.y)
	local x = mx - self.start_x

	for i, v in ipairs(selection.list) do
		local n = #v.verts
		local t_end = self.prev_state[i].verts[n][1]

		assert(t_end > 0)

		local t_move = t_end + x
		t_move = math.max(t_min, t_move)

		for j in ipairs(v.verts) do
			-- proportion to shift
			local s = self.prev_state[i].verts[j][1] / t_end
			v.verts[j][1] = s * t_move
		end
	end
end

function drag_end:mousereleased(canvas)
	if self.edit then
		local c = command.NoteUpdate.new(self.prev_state, selection.list)
		command.register(c)
		self.prev_state = nil

		return true
	end
end

function drag_end:draw(canvas) end

return drag_end
