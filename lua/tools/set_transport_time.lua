local engine = require("engine")

local set_transport_time = {}

function set_transport_time:mousepressed(canvas)
	self.do_start = false
	self.time = 0
	if engine.playing then
		self.do_start = true
		engine.stop()
	end
end

function set_transport_time:mousedown(canvas)
	-- if engine was started during drag, cancel
	if engine.playing then
		self.do_start = false
	else
		local mx, _ = canvas:getMouse()

		self.time = canvas.transform:time_inv(mx)

		project.transport.start_time = self.time
		engine.seek(self.time)
	end
end

function set_transport_time:mousereleased(canvas)
	if self.do_start then
		engine.start()
	end
end

function set_transport_time:draw(canvas) end

return set_transport_time
