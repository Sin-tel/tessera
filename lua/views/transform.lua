local Transform = {}
Transform.__index = Transform

function Transform.new(ch_index)
	local self = setmetatable({}, Transform)

	self.sx = 90
	self.sy = -12

	self.ox = 200
	self.oy = 900

	self.ox_ = self.ox
	self.oy_ = self.oy

	self.sx_ = self.sx
	self.sy_ = self.sy

	return self
end

function Transform:time(t)
	return t * self.sx + self.ox
end

function Transform:time_inv(t)
	return (t - self.ox) / self.sx
end

function Transform:pitch(p)
	return p * self.sy + self.oy
end

function Transform:pitch_inv(p)
	return (p - self.oy) / self.sy
end

function Transform:inverse(x, y)
	return (x - self.ox) / self.sx, (y - self.oy) / self.sy
end

function Transform:update()
	-- interpolation update
	local sf = 0.5

	self.ox = self.ox + sf * (self.ox_ - self.ox)
	self.oy = self.oy + sf * (self.oy_ - self.oy)
	self.sx = self.sx + sf * (self.sx_ - self.sx)
	self.sy = self.sy + sf * (self.sy_ - self.sy)
end

function Transform:zoom_x(center, zoom_factor)
	self.sx_ = self.sx_ * zoom_factor
	self.ox_ = self.ox_ + (center - self.ox_) * (1 - zoom_factor)
end

function Transform:zoom_y(center, zoom_factor)
	self.sy_ = self.sy_ * zoom_factor
	self.oy_ = self.oy_ + (center - self.oy_) * (1 - zoom_factor)
end

function Transform:pan(x, y)
	-- instant update
	self.ox = x
	self.oy = y
	self.ox_ = self.ox
	self.oy_ = self.oy
end

return Transform
