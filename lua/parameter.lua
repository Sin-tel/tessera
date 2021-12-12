Parameter = {}

function Parameter:new(name, tbl)
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.t = tbl.t
	new.name = name or "Value"
	new.centered = tbl.centered

	if new.t == "dB" then
		new.v = from_dB(tbl.default)
		new.default = from_dB(tbl.default)
		new.min = 0 --from_dB(tbl.min or -math.huge)
		new.max = (tbl.max or 0)

		assert(new.default <= from_dB(new.max))
	else
		new.v = tbl.default
		new.default = tbl.default
		new.min = tbl.min or 0
		new.max = tbl.max or 1
		assert(new.min < new.max)
		assert(new.min <= new.default)
		assert(new.default <= new.max)
	end

	

	return new	
end

function Parameter:set(x)
	self.v = clamp(x, self.min, self.max)
end

function Parameter:reset()
	self.v = self.default
end

function Parameter:setNormalized(x)
	local x = clamp(x, 0, 1)
	if self.t == "dB" then
		x = curve_dB(x, self.max)
		self.v = x
	else
		self.v = x * (self.max - self.min) + self.min
	end
end

function Parameter:getNormalized(x)
	if self.t == "dB" then
		return curve_dB_inv(self.v, self.max)
	else
		return (self.v - self.min) / (self.max - self.min)
	end
	
end

function Parameter:getDisplay()
	if self.t == "dB" then
		return string.format("%0.1f dB", to_dB(self.v))
		-- return string.format("%0.3f", self.v)
	else
		return string.format("%0.3f", self.v)
	end
end