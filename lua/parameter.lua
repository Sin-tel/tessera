Parameter = {}

function Parameter:new(name, tbl)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.t = tbl.t
	new.name = name or "Value"
	new.centered = tbl.centered

	new.dirty = true

	if new.t == "dB" then
		new.v = from_dB(tbl.default)
		new.default = from_dB(tbl.default)
		new.min = 0 --from_dB(tbl.min or -math.huge)
		new.max = (tbl.max or 0)

		assert(new.default <= from_dB(new.max))
	else
		new.min = tbl.min or 0
		new.max = tbl.max or 1
		local default = tbl.default or 0.5 * (new.max + new.min)
		new.v = default
		new.default = default
		new.fmt = tbl.fmt or "%0.3f"
		assert(new.min < new.max)
		assert(new.min <= new.default)
		assert(new.default <= new.max)
	end

	return new
end

-- function Parameter:setRaw(x)
-- 	self.v = x
-- 	self.dirty = true
-- end

function Parameter:reset()
	self.v = self.default
	self.dirty = true
end

function Parameter:setNormalized(x)
	local x = clamp(x, 0, 1)
	if self.t == "dB" then
		x = curve_dB(x, self.max)
		self.v = x
	else
		self.v = x * (self.max - self.min) + self.min
	end
	self.dirty = true
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
	else
		return string.format(self.fmt, self.v)
	end
end
