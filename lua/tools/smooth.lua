local brush = require("tools/brush")

local smooth = brush.new("smooth", 30)

-- Each vert moves towards the middle of its neighbours, using only old values so verts can't cross.
-- The ends are fixed in time and pressure, their pitch follows the next vert.
function smooth:stroke(note, weights)
	local verts = note.verts
	local n = #verts

	local d_first = (verts[2][2] - verts[1][2]) * weights[1]
	local d_last = (verts[n - 1][2] - verts[n][2]) * weights[n]

	local l_t, l_p, l_w = verts[1][1], verts[1][2], verts[1][3]
	for i = 2, n - 1 do
		local v = verts[i]
		local r = verts[i + 1]
		local w = weights[i]
		local t, p, pres = v[1], v[2], v[3]

		v[1] = t + (0.5 * (l_t + r[1]) - t) * w
		v[2] = p + (0.5 * (l_p + r[2]) - p) * w
		v[3] = pres + (0.5 * (l_w + r[3]) - pres) * w

		l_t, l_p, l_w = t, p, pres
	end

	verts[1][2] = verts[1][2] + d_first
	verts[n][2] = verts[n][2] + d_last
end

return smooth
