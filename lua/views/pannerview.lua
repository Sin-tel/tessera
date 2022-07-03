PannerView = View:derive("Panning")

function PannerView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()


	local radius = math.min(w/2,h)*0.95
	local radius2 = radius * 0.15

	if radius > 32 then

		local cx = w * 0.5
		local cy = h * 0.98

		mx = mx - cx
		my = my - cy
		local a = math.atan2(my, mx)

		local d = length(mx, my)
		d = clamp(d,radius2,radius)
		if a > math.pi*0.5 then a = -math.pi end
		a = clamp(a,-math.pi,0)

		mx = d*math.cos(a) + cx
		my = d*math.sin(a) + cy

		
		love.graphics.setColor(Theme.widget_line)
		-- love.graphics.setColor(White)
		love.graphics.arc("line", "open", cx, cy, radius, 0, -math.pi)
		love.graphics.arc("line", "open", cx, cy, radius2, 0, -math.pi)
		love.graphics.line(cx - radius2, cy, cx - radius, cy)
		love.graphics.line(cx + radius2, cy, cx + radius, cy)

		love.graphics.setColor(Theme.ui_text)
		if self.box.focus then
			love.graphics.ellipse("fill", mx, my, 5)
			nd = (d-radius2) / (radius - radius2)
			nd = 17.31234*math.log(1-nd) -- magic formula
			
			love.graphics.print(string.format("%0.1f dB", (nd)), math.floor(mx),math.floor( my-24))
			-- love.graphics.print(string.format("%0.5f", from_dB(nd)), mx, my-24)
		end
	end
end
