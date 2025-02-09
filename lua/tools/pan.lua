local pan = {}

function pan:mousepressed(song)
	self.drag_ix = song.transform.ox
	self.drag_iy = song.transform.oy
end

function pan:mousedown(song)
	local px = self.drag_ix + mouse.dx
	local py = self.drag_iy + mouse.dy
	song.transform:pan(px, py)
end

function pan:mousereleased(song) end

function pan:draw(song) end

return pan
