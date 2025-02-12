local SliderValue = {}
SliderValue.__index = SliderValue

-- TODO: bipolar db scale
function SliderValue.new(options)
	local self = setmetatable({}, SliderValue)

	self.t = options.t
	self.centered = options.centered

	if self.t == "dB" then
		self.default = util.from_dB(options.default or 0)
		self.min = 0
		self.max = (options.max or 0)
		self.fmt = options.fmt or "%0.1f dB"

		assert(options.min == nil)
		assert(self.default <= util.from_dB(self.max))
	elseif self.t == "log" then
		self.min = options.min
		self.max = options.max
		local default = options.default or math.sqrt(self.min * self.max)
		self.default = default
		self.fmt = options.fmt or "%0.2f"
		assert(self.min <= self.max)
		assert(self.min > 0)
		assert(self.max > 0)
		assert(self.min <= self.default)
		assert(self.default <= self.max)
	else
		self.min = options.min or 0
		self.max = options.max or 1
		self.step = options.step
		local default = options.default or 0.5 * (self.min + self.max)
		self.default = default
		self.fmt = options.fmt or "%0.2f"
		assert(self.min < self.max)
		assert(self.min <= self.default)
		assert(self.default <= self.max)
	end

	return self
end

function SliderValue:fromNormal(value)
	local x = util.clamp(value, 0, 1)
	if self.t == "dB" then
		x = util.curve_dB(x, self.max)
		return x
	elseif self.t == "log" then
		return math.exp(x * math.log(self.max / self.min) + math.log(self.min))
	else
		local v = x * (self.max - self.min) + self.min
		if self.step then
			v = math.floor(v / self.step) * self.step
		end
		return v
	end
end

function SliderValue:toNormal(value)
	if self.t == "dB" then
		return util.curve_dB_inv(value, self.max)
	elseif self.t == "log" then
		return math.log(value / self.min) / math.log(self.max / self.min)
	else
		return (value - self.min) / (self.max - self.min)
	end
end

function SliderValue:toString(value)
	local display = value

	if self.t == "dB" then
		display = util.to_dB(value)
	end

	if self.fmt == "Hz" then
		if display < 100 then
			return string.format("%.1f Hz", display)
		elseif display < 1000 then
			return string.format("%.0f Hz", display)
		elseif display < 10000 then
			return string.format("%.2f kHz", display / 1000)
		else
			return string.format("%.1f kHz", display / 1000)
		end
	elseif self.fmt == "ms" or self.fmt == "s" then
		if self.fmt == "s" then
			display = display * 1000
		end

		if display < 10 then
			return string.format("%.1f ms", display)
		elseif display < 1000 then
			return string.format("%.0f ms", display)
		elseif display < 10000 then
			return string.format("%.2f s", display / 1000)
		else
			return string.format("%.1f s", display / 1000)
		end
	else
		return string.format(self.fmt, display)
	end
end

return SliderValue
