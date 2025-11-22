local engine = require("engine")

local set_transport_time = {}

function set_transport_time:mousepressed(canvas) end

function set_transport_time:mousedown(canvas)
	local mx, _ = canvas:getMouse()

	local new_time = canvas.transform:time_inv(mx)

	project.transport.start_time = new_time
	engine.seek(new_time)
end

function set_transport_time:mousereleased(canvas) end

function set_transport_time:draw(canvas) end

return set_transport_time
