-- Raises pressure under the brush, ctrl lowers it. Works in logit space, so pressure eases towards 0 and 1
-- (the last 1% is linear, so held long enough it does reach them).
-- With nothing selected it works on the whole active channel.

local brush = require("tools/brush")
local smooth = require("tools/smooth")

local dodge_burn = brush.new("dodge_burn", 30)
dodge_burn.modifiers = { shift = smooth }
dodge_burn.whole_channel = true

-- logit change per frame under the mouse
local STRENGTH = 0.06
-- logit is infinite at 0 and 1: closer than this, verts move by the step they'd get at EPS
local EPS = 0.01

function dodge_burn:stroke(note, weights)
	local d = (modifier_keys.ctrl and -1 or 1) * STRENGTH
	for i, v in ipairs(note.verts) do
		local w = util.clamp(v[3], EPS, 1 - EPS)
		local new = 1 / (1 + math.exp(-(math.log(w / (1 - w)) + d * weights[i])))
		v[3] = util.clamp(v[3] + new - w, 0, 1)
	end
end

return dodge_burn
