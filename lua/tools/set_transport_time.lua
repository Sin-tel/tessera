local engine = require("engine")

local set_transport_time = {}

function set_transport_time:mousepressed(canvas)
	self.was_playing = false
	self.time = 0
	if engine.playing then
		self.was_playing = true
		engine.stop()
	end
end

function set_transport_time:mousedown(canvas)
	local mx, _ = canvas:getMouse()

	self.time = canvas.transform:time_inv(mx)

	project.transport.start_time = self.time
	engine.seek(self.time)
end

function set_transport_time:mousereleased(canvas)
	if self.was_playing then
		engine.start()
	end
end

function set_transport_time:draw(canvas) end

return set_transport_time
