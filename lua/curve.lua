-- Operations on a note's verts: { t_offset, pitch_offset, pressure }, sorted by time, first at t = 0.

local curve = {}

-- what simplify may change, small enough not to hear
local PITCH_TOL = 0.02
local PRESSURE_TOL = 0.01

-- Ramer-Douglas-Peucker on pitch and pressure. Always keeps the ends.
function curve.simplify(verts)
	local n = #verts
	if n <= 2 then
		return verts
	end

	local keep = { [1] = true, [n] = true }
	local stack = { 1, n }
	while #stack > 0 do
		local i2 = table.remove(stack)
		local i1 = table.remove(stack)
		local a, b = verts[i1], verts[i2]

		local max_err = 1
		local index
		for i = i1 + 1, i2 - 1 do
			local v = verts[i]
			local s = (v[1] - a[1]) / (b[1] - a[1])
			local err = math.abs(v[2] - util.lerp(a[2], b[2], s)) / PITCH_TOL
				+ math.abs(v[3] - util.lerp(a[3], b[3], s)) / PRESSURE_TOL
			if err > max_err then
				max_err = err
				index = i
			end
		end

		if index then
			keep[index] = true
			table.insert(stack, i1)
			table.insert(stack, index)
			table.insert(stack, index)
			table.insert(stack, i2)
		end
	end

	local out = {}
	for i, v in ipairs(verts) do
		if keep[i] then
			table.insert(out, v)
		end
	end
	return out
end

-- Splits segments so verts are at most `max_dt` apart. The curve itself doesn't change.
function curve.subdivide(verts, max_dt)
	local out = { verts[1] }
	for i = 2, #verts do
		local a, b = verts[i - 1], verts[i]
		local parts = math.ceil((b[1] - a[1]) / max_dt)
		for k = 1, parts - 1 do
			local s = k / parts
			table.insert(out, { util.lerp(a[1], b[1], s), util.lerp(a[2], b[2], s), util.lerp(a[3], b[3], s) })
		end
		table.insert(out, b)
	end
	return out
end

return curve
