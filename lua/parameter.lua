local Parameter = {}

-- TODO: lists, on/off
-- TODO: bipolar db scale (rename db to gain or sth)
function Parameter:new(name, tbl)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.t = tbl.t
	new.name = name or "Value"
	new.centered = tbl.centered

	new.dirty = true

	if new.t == "dB" then
		new.v = util.from_dB(tbl.default)
		new.default = util.from_dB(tbl.default)
		new.min = 0
		new.max = (tbl.max or 0)
		new.fmt = tbl.fmt or "%0.1f dB"

		assert(tbl.min == nil)
		assert(new.default <= util.from_dB(new.max))
	elseif new.t == "log" then
		new.min = tbl.min
		new.max = tbl.max
		local default = tbl.default or math.sqrt(new.min * new.max)
		new.v = default
		new.default = default
		new.fmt = tbl.fmt or "%0.2f"
		assert(new.min < new.max)
		assert(new.min > 0)
		assert(new.max > 0)
		assert(new.min <= new.default)
		assert(new.default <= new.max)
	else
		new.min = tbl.min or 0
		new.max = tbl.max or 1
		local default = tbl.default or 0.5 * (new.min + new.max)
		new.v = default
		new.default = default
		new.fmt = tbl.fmt or "%0.2f"
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

function Parameter:setNormalized(value)
	local x = util.clamp(value, 0, 1)
	if self.t == "dB" then
		x = util.curve_dB(x, self.max)
		self.v = x
	elseif self.t == "log" then
		self.v = math.exp(x * math.log(self.max / self.min) + math.log(self.min))
	else
		self.v = x * (self.max - self.min) + self.min
	end
	self.dirty = true
end

function Parameter:getNormalized()
	if self.t == "dB" then
		return util.curve_dB_inv(self.v, self.max)
	elseif self.t == "log" then
		return math.log(self.v / self.min) / math.log(self.max / self.min)
	else
		return (self.v - self.min) / (self.max - self.min)
	end
end

function Parameter:getDisplay()
	if self.t == "dB" then
		return string.format(self.fmt, util.to_dB(self.v))
	else
		if self.fmt == "Hz" then
			if self.v < 1000 then
				return string.format("%.0f Hz", self.v)
			else
				return string.format("%.2f kHz", self.v / 1000)
			end
		else
			return string.format(self.fmt, self.v)
		end
	end
end

return Parameter
