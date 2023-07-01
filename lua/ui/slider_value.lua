local SliderValue = {}

-- TODO: bipolar db scale (rename db to gain or sth)
-- TODO: initial value seperate from default value
function SliderValue:new(options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.t = options.t
	new.centered = options.centered

	new.dirty = true

	if new.t == "dB" then
		new.v = util.from_dB(options.default)
		new.default = util.from_dB(options.default)
		new.min = 0
		new.max = (options.max or 0)
		new.fmt = options.fmt or "%0.1f dB"

		assert(options.min == nil)
		assert(new.default <= util.from_dB(new.max))
	elseif new.t == "log" then
		new.min = options.min
		new.max = options.max
		local default = options.default or math.sqrt(new.min * new.max)
		new.v = default
		new.default = default
		new.fmt = options.fmt or "%0.2f"
		assert(new.min <= new.max)
		assert(new.min > 0)
		assert(new.max > 0)
		assert(new.min <= new.default)
		assert(new.default <= new.max)
	else
		new.min = options.min or 0
		new.max = options.max or 1
		new.step = options.step
		local default = options.default or 0.5 * (new.min + new.max)
		new.v = default
		new.default = default
		new.fmt = options.fmt or "%0.2f"
		assert(new.min < new.max)
		assert(new.min <= new.default)
		assert(new.default <= new.max)
	end

	return new
end

function SliderValue:reset()
	self.v = self.default
	self.dirty = true
end

function SliderValue:setNormalized(value)
	local x = util.clamp(value, 0, 1)
	if self.t == "dB" then
		x = util.curve_dB(x, self.max)
		self.v = x
	elseif self.t == "log" then
		self.v = math.exp(x * math.log(self.max / self.min) + math.log(self.min))
	else
		local v = x * (self.max - self.min) + self.min
		if self.step then
			v = math.floor(v / self.step) * self.step
		end
		self.v = v
	end
	self.dirty = true
end

function SliderValue:getNormalized()
	if self.t == "dB" then
		return util.curve_dB_inv(self.v, self.max)
	elseif self.t == "log" then
		return math.log(self.v / self.min) / math.log(self.max / self.min)
	else
		return (self.v - self.min) / (self.max - self.min)
	end
end

function SliderValue:asString()
	if self.t == "dB" then
		return string.format(self.fmt, util.to_dB(self.v))
	else
		if self.fmt == "Hz" then
			if self.v < 100 then
				return string.format("%.1f Hz", self.v)
			elseif self.v < 1000 then
				return string.format("%.0f Hz", self.v)
			elseif self.v < 10000 then
				return string.format("%.2f kHz", self.v / 1000)
			else
				return string.format("%.1f kHz", self.v / 1000)
			end
		elseif self.fmt == "ms" then
			if self.v < 1000 then
				return string.format("%.0f ms", self.v)
			elseif self.v < 10000 then
				return string.format("%.2f s", self.v / 1000)
			else
				return string.format("%.1f s", self.v / 1000)
			end
		else
			return string.format(self.fmt, self.v)
		end
	end
end

return SliderValue
