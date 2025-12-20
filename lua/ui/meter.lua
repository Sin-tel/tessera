local Ui = require("ui/ui")

local Meter = {}
Meter.__index = Meter

-- obj is any object with 'meter_l' and 'meter_r' field
function Meter.new(obj)
	local self = setmetatable({}, Meter)

	self.obj = obj

	return self
end

function Meter:update(ui)
	local x, y, w, h = ui:next()

	ui:push_draw(self.draw, { self, ui, x, y, w, h })

	-- no interaction
	return false
end

function Meter:draw(ui, x, y, w, h)
	local ml = self.obj.meter_l
	local mr = self.obj.meter_r

	local cl = util.meter_color(ml)
	local cr = util.meter_color(mr)

	local wl = util.clamp((util.to_dB(ml) + 80) / 80, 0, 1)
	local wr = util.clamp((util.to_dB(mr) + 80) / 80, 0, 1)

	local h1 = Ui.scale(16)
	local y1 = y + 0.5 * (h - h1)
	local h2 = 0.5 * h1
	tessera.graphics.set_color(theme.bg_nested)
	tessera.graphics.rectangle("fill", x, y1, w, h1)
	if wl > 0 then
		tessera.graphics.set_color(cl)
		tessera.graphics.rectangle("fill", x, y1, w * wl, h2 - 1)
	end
	if wr > 0 then
		tessera.graphics.set_color(cr)
		tessera.graphics.rectangle("fill", x, y1 + h2, w * wr, h2)
	end
	tessera.graphics.set_color(theme.line)
	tessera.graphics.rectangle("line", x, y1 - 0.5, w, h1, 2)
end

return Meter
