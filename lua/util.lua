local log = require("log")
local serialize = require("lib.serialize")

local util = {}

local EPSILON = 1e-5

function util.find(list, value)
	for i, v in ipairs(list) do
		if v == value then
			return i
		end
	end
end

function util.map(list, fn)
	local new = {}
	for i, v in ipairs(list) do
		new[i] = fn(v)
	end
	return new
end

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

function util.unpack_r(rect)
	return rect.x, rect.y, rect.w, rect.h
end

function util.hit(rect, mx, my)
	local x, y, w, h = util.unpack_r(rect)
	return mx >= x - 1 and my >= y - 1 and mx <= x + w + 2 and my <= y + h + 2
end

function util.segment_dist_sq(px, py, x1, y1, x2, y2)
	local l2 = (x1 - x2) ^ 2 + (y1 - y2) ^ 2
	if l2 < EPSILON then
		return (px - x1) ^ 2 + (py - y1) ^ 2
	end

	local t = ((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / l2
	t = math.max(0, math.min(1, t))

	return (px - (x1 + t * (x2 - x1))) ^ 2 + (py - (y1 + t * (y2 - y1))) ^ 2
end

function util.segment_intersect(p1x, p1y, p2x, p2y, p3x, p3y, p4x, p4y)
	local d = (p2x - p1x) * (p4y - p3y) - (p2y - p1y) * (p4x - p3x)
	if d < EPSILON then
		return false
	end

	local t = ((p3x - p1x) * (p4y - p3y) - (p3y - p1y) * (p4x - p3x)) / d
	local u = ((p3x - p1x) * (p2y - p1y) - (p3y - p1y) * (p2x - p1x)) / d

	return t >= 0 and t <= 1 and u >= 0 and u <= 1
end

function util.line_box_intersect(x1, y1, x2, y2, bx1, by1, bx2, by2)
	-- check if either endpoint is inside the box
	if
		(x1 >= bx1 and x1 <= bx2 and y1 >= by1 and y1 <= by2) or (x2 >= bx1 and x2 <= bx2 and y2 >= by1 and y2 <= by2)
	then
		return true
	end

	-- check intersection with each of the four box edges
	return util.segment_intersect(x1, y1, x2, y2, bx1, by1, bx2, by1) -- top
		or util.segment_intersect(x1, y1, x2, y2, bx2, by1, bx2, by2) -- right
		or util.segment_intersect(x1, y1, x2, y2, bx2, by2, bx1, by2) -- bottom
		or util.segment_intersect(x1, y1, x2, y2, bx1, by2, bx1, by1) -- left
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
-- TODO: simplify: this is just a power law!
-- halfway 18dB ~ x^3
function util.curve_dB(x, max)
	return util.from_dB(curve_param * math.log(x) + (max or 0))
end

function util.curve_dB_inv(x, max)
	return math.exp((util.to_dB(x) - (max or 0)) / curve_param)
end

function util.ratio(r)
	return 12.0 * math.log(r) / math.log(2)
end

-- TODO: make this configurable (per intrument?)
-- 0.01 = 40dB dynamic range
-- 0.02 = 34dB dynamic range
-- 0.05 = 26dB dynamic range
-- 0.10 = 20dB dynamic range
local VEL_MIN = 0.05
local LOG_RANGE = -math.log(VEL_MIN)

function util.velocity_curve(x)
	local v = x ^ 0.8
	local out = VEL_MIN * math.exp(LOG_RANGE * v)
	return out
end

local METER_CLIP_COLOR = { 1.00, 0.30, 0.20 }

function util.meter_color(x, darken)
	if x > 1 then
		return METER_CLIP_COLOR
	else
		assert(x >= 0)
		local v = 0.7
		if darken then
			-- local x_log = math.max((util.to_dB(x) + 60) / 60, 0)
			local x_log = x ^ 0.25
			v = 0.9 * x_log
			v = util.clamp(v, 0, 1)
		end
		return tessera.graphics.get_color_hsv((140 - 110 * x * x) / 360, 0.7, v)
	end
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

local function copy_defaults(t, default)
	if t == nil or type(t) ~= "table" then
		return clone(default)
	end
	if type(default) ~= "table" then
		return t
	end

	for key, default_value in pairs(default) do
		if t[key] == nil then
			t[key] = clone(default_value)
		elseif type(default_value) == "table" and type(t[key]) == "table" then
			-- recursive merge
			copy_defaults(t[key], default_value)
		end
	end
end

util.copy_defaults = copy_defaults

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

function util.dump(t)
	if type(t) == "table" then
		return serialize(t)
	elseif type(t) == "string" then
		return '"' .. t .. '"'
	else
		return tostring(t)
	end
end

function util.pprint(t)
	print(util.dump(t))
end

function util.capitalize(s)
	return s:sub(1, 1):upper() .. s:sub(2):lower()
end

function util.version_compatible(current, other)
	-- we guarantee backward compatibility but not forward.
	if current.MAJOR == 0 and other.MAJOR == 0 then
		if current.MINOR == other.MINOR and current.PATCH >= other.PATCH then
			return true
		end
	else
		return current.MAJOR == other.MAJOR and current.MINOR >= other.MINOR
	end

	return false
end

function util.version_str(version)
	return string.format("v%d.%d.%d", version.MAJOR, version.MINOR, version.PATCH)
end

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

function util.file_exists(filename)
	local f = io.open(filename, "r")
	if f ~= nil then
		io.close(f)
		return true
	else
		return false
	end
end

return util
