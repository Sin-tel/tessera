local pan = {}

function pan:mousepressed(canvas)
	self.drag_ix = canvas.transform.ox
	self.drag_iy = canvas.transform.oy
end

function pan:mousedown(canvas)
	local px = self.drag_ix + mouse.dx
	local py = self.drag_iy + mouse.dy
	canvas.transform:pan(px, py)
end

function pan:update()
	mouse:set_cursor("grab")
end

function pan:mousereleased(canvas) end

function pan:draw(canvas) end

return pan
