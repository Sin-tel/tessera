local util = require("util")
local tuning = require("tuning")

local adjust_velocity = {}

adjust_velocity.start_y = 0
adjust_velocity.selection = {}

-- Sigmoid function to preserve velocity in [0, 1]
local function sigmoid(x)
	return 1 / (1 + math.exp(-x))
end

local function sigmoid_inv(x)
	return math.log(x / (1 - x))
end

function adjust_velocity:mousepressed(canvas)
	self.start_y = mouse.y
	self.selection = util.clone(selection.list)
end

function adjust_velocity:mousedown(canvas)
	local y = -0.03 * (mouse.y - self.start_y)

	for i, v in ipairs(selection.list) do
		local x = sigmoid_inv(self.selection[i].vel)
		x = sigmoid(x + y)
		x = util.clamp(x, 0.001, 0.999)
		v.vel = x
	end
end

function adjust_velocity:mousereleased(canvas) end

function adjust_velocity:draw(canvas) end

return adjust_velocity
