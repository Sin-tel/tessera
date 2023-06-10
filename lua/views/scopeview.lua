local View = require("view")
local backend = require("backend")

local scopeView = View:derive("Scope")

scopeView.spectrum = { 0 }

function scopeView:draw()
	local w, h = self:getDimensions()

	-- @todo make sure this only gets called once
	local spec = backend:get_spectrum()
	if spec then
		self.spectrum = spec
	end

	love.graphics.setColor(theme.ui_text)
	local tx = w * 0.95
	local ty = h * 0.1
	local sx = (w * 0.9) / #self.spectrum
	local sy = h * 0.4

	local n = #self.spectrum

	for i = 1, n - 1 do
		local x1 = 300 * (math.log(i / n))
		local x2 = 300 * (math.log((i + 1) / n))

		local y1 = 0.2 * (math.log(self.spectrum[i]))
		local y2 = 0.2 * (math.log(self.spectrum[i + 1]))

		love.graphics.line(tx + x1 * sx, ty - sy * y1, tx + x2 * sx, ty - sy * y2)
	end

	-- print(#spectrum, spectrum[0])
end

return scopeView
