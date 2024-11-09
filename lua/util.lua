local log = require("log")
local Ui = require("ui/ui")

local util = {}

function util.lerp(a, b, t)
	return a + (b - a) * util.clamp(t, 0, 1)
end

function util.towards(a, b, t)
	if math.abs(b - a) < t then
		return b
	else
		return a + t * util.sign(b - a)
	end
end

function util.sign(x)
	if x > 0 then
		return 1
	elseif x < 0 then
		return -1
	else
		return 0
	end
end

function util.clamp(x, min, max)
	return math.min(math.max(x, min), max)
end

function util.dist(x1, y1, x2, y2)
	return math.sqrt((x1 - x2) ^ 2 + (y1 - y2) ^ 2)
end

function util.length(x, y)
	return math.sqrt(x ^ 2 + y ^ 2)
end

function util.from_dB(x)
	return 10.0 ^ (x / 20.0)
end

function util.to_dB(x)
	return 20.0 * math.log10(x)
end

-- set dB at halfway point
-- 12 or 18 is good
local curve_param = -18 / math.log(0.5)

function util.curve_dB(x, max)
	return util.from_dB(curve_param * math.log(x) + (max or 0))
end

function util.curve_dB_inv(x, max)
	return math.exp((util.to_dB(x) - (max or 0)) / curve_param)
end

function util.ratio(r)
	return 12.0 * math.log(r) / math.log(2)
end

-- clone a simple, non-recursive data table
local function clone(orig, seen)
	seen = seen or {}
	local copy
	if type(orig) == "table" then
		assert(not seen[orig])
		copy = {}
		seen[orig] = true
		for k, v in pairs(orig) do
			assert(type(k) ~= "table")
			copy[k] = clone(v, seen)
		end
	else -- number, string, boolean, etc
		copy = orig
	end
	return copy
end

util.clone = clone

function util.drawText(str, x, y, w, h, align, pad)
	align = align or "left"

	assert(type(str) == "string", type(str))

	if pad then
		local p = Ui.DEFAULT_PAD
		x = x + p
		w = w - 2 * p
	end

	local f = love.graphics.getFont()
	local str2 = str
	local fh = f:getHeight()

	local oy = 0.5 * (h - fh)

	local l = str:len()
	local strip = false
	while true do
		local fw = f:getWidth(str2)
		if fw < w then
			if align == "left" or strip then
				love.graphics.print(str2, math.floor(x), math.floor(y + oy))
			elseif align == "center" then
				love.graphics.print(str2, math.floor(x + 0.5 * (w - fw)), math.floor(y + oy))
			elseif align == "right" then
				love.graphics.print(str2, math.floor(x + w - fw), math.floor(y + oy))
			end
			break
		end
		l = l - 1
		if l <= 0 then
			break
		end
		str2 = str:sub(1, l) .. "..."
		strip = true
	end
end

function util.average(t)
	local n = #t
	if n == 0 then
		return 0
	end
	local sum = 0
	for _, v in ipairs(t) do
		sum = sum + v
	end
	return sum / n
end

local function pprint(t, indent)
	indent = indent or 0
	if type(t) == "table" then
		for k, v in pairs(t) do
			if type(v) == "table" then
				print(string.rep("  ", indent) .. tostring(k) .. ":")
				pprint(v, indent + 1)
			else
				local s = tostring(v)
				if type(v) == "string" then
					s = '"' .. s .. '"'
				end
				print(string.rep("  ", indent) .. tostring(k) .. ": " .. s)
			end
		end
	else
		print(t)
	end
end

util.pprint = pprint

function util.writefile(filename, contents)
	local file, err = io.open(filename, "w")
	if err then
		log.error(err)
		return err
	end
	file:write(contents)
	file:close()
end

function util.readfile(filename)
	local file = assert(io.open(filename, "r"))
	local content = file:read("*a")
	file:close()
	return content
end

function util.fileExists(filename)
	local f = io.open(filename, "r")
	if f ~= nil then
		io.close(f)
		return true
	else
		return false
	end
end

return util
