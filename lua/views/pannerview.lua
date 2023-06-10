local View = require("view")

local pannerView = View:derive("Panning")

function pannerView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	local radius = math.min(w / 2, h) * 0.95
	local radius2 = radius * 0.15

	if radius > 32 then
		local cx = w * 0.5
		local cy = h * 0.98

		mx = mx - cx
		my = my - cy
		local a = math.atan2(my, mx)

		local d = util.length(mx, my)
		d = util.clamp(d, radius2, radius)
		if a > math.pi * 0.5 then
			a = -math.pi
		end
		a = util.clamp(a, -math.pi, 0)

		mx = d * math.cos(a) + cx
		my = d * math.sin(a) + cy

		love.graphics.setColor(theme.widget_line)
		-- love.graphics.setColor(White)
		love.graphics.arc("line", "open", cx, cy, radius, 0, -math.pi)
		love.graphics.arc("line", "open", cx, cy, radius2, 0, -math.pi)
		love.graphics.line(cx - radius2, cy, cx - radius, cy)
		love.graphics.line(cx + radius2, cy, cx + radius, cy)

		love.graphics.setColor(theme.ui_text)
		if self.box.focus then
			love.graphics.ellipse("fill", mx, my, 5)
			local nd = (d - radius2) / (radius - radius2)
			nd = 17.31234 * math.log(1 - nd) -- magic formula

			love.graphics.print(string.format("%0.1f dB", nd), math.floor(mx), math.floor(my - 24))
			-- love.graphics.print(string.format("%0.5f", from_dB(nd)), mx, my-24)
		end

		for _, ch in ipairs(channelHandler.list) do
			for _, fx in ipairs(ch.effects) do
				local gain = fx.parameters[1].v
				local pan = fx.parameters[2].v

				local dist = 1.0 - math.exp(util.to_dB(gain) / 17.31234)
				dist = dist * (radius - radius2) + radius2

				-- local a = 2*math.asin(pan) / math.pi
				local angle = pan
				angle = 0.5 * ((angle - 1) * math.pi)

				local x = dist * math.cos(angle) + cx
				local y = dist * math.sin(angle) + cy

				if ch == selection.channel then
					love.graphics.ellipse("fill", x, y, 6)
				end

				love.graphics.ellipse("line", x, y, 6)
			end
		end
	end
end

return pannerView
