local SliderValue = {}

-- TODO: bipolar db scale
function SliderValue:new(options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.t = options.t
	new.centered = options.centered

	new.dirty = true

	if new.t == "dB" then
		new.default = util.from_dB(options.default or 0)
		new.min = 0
		new.max = (options.max or 0)
		new.fmt = options.fmt or "%0.1f dB"

		assert(options.min == nil)
		assert(new.default <= util.from_dB(new.max))
	elseif new.t == "log" then
		new.min = options.min
		new.max = options.max
		local default = options.default or math.sqrt(new.min * new.max)
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
		new.default = default
		new.fmt = options.fmt or "%0.2f"
		assert(new.min < new.max)
		assert(new.min <= new.default)
		assert(new.default <= new.max)
	end

	return new
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
