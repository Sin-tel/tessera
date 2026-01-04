-- TODO: undo

local tempo = {}

function tempo:mousepressed(canvas)
	self.ix, self.iy = mouse.x, mouse.y
	self.edit = false
	self.prev_state = util.clone(project.time)
end

function tempo:mousedown(canvas)
	local x = mouse.dx
	local y = mouse.dy

	-- if modifier_keys.shift then
	-- 	-- Constrain to one axis
	-- 	if math.abs(self.ix - mouse.x) < math.abs(self.iy - mouse.y) then
	-- 		x = 0
	-- 	else
	-- 		y = 0
	-- 	end
	-- end

	if mouse.drag then
		self.edit = true
		project.time[1][1] = self.prev_state[1][1] + x / canvas.transform.sx
		project.time[1][2] = self.prev_state[1][2] * math.exp(-0.001 * y)
	end
end

function tempo:mousereleased(canvas)
	if self.edit then
		local c = command.TimeUpdate.new(self.prev_state, project.time)
		command.register(c)
		self.prev_state = nil
		return true
	end
end

function tempo:draw(canvas) end

return tempo
