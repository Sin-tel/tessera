-- TODO: undo

local tempo = {}

function tempo:mousepressed(canvas)
	self.ix, self.iy = mouse.x, mouse.y
	self.time_start = project.time[1][1]
	self.tempo_start = project.time[1][2]
end

function tempo:mousedown(canvas)
	local mx, my = mouse.x, mouse.y

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
		project.time[1][1] = self.time_start + x / canvas.transform.sx
		project.time[1][2] = self.tempo_start * math.exp(-0.001 * y)
	end
end

function tempo:mousereleased(canvas) end

function tempo:draw(canvas) end

return tempo
