local View = require("view")
local backend = require("backend")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")
local Scope = View:derive("Scope")

function Scope:new(spectrum)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	self.index = 1
	if spectrum then
		self.index = 2
	end

	new.ui = Ui:new(new)

	new.selector = widgets.Selector:new({ "scope", "spectrum" })

	self.lines = {}
	for i = 1, 7 do
		table.insert(self.lines, 4 ^ (1 - i))
	end

	self.average = {}
	for i = 1, 4096 do
		self.average[i] = 0
	end

	return new
end

function Scope:update()
	self.ui:startFrame()
	self.ui.layout:col(200)
	self.selector:update(self.ui, self, "index")
	self.ui:endFrame()
end

function Scope:draw()
	local w, h = self:getDimensions()

	if self.index == 2 then
		local spectrum = backend:getSpectrum()
		if spectrum then
			local n = #spectrum

			for i = 1, n do
				self.average[i] = self.average[i] + 1.0 * (spectrum[i] - self.average[i])
			end

			local tx = w * 0.95
			local ty = h * 0.1
			local sx = (w * 0.9) / n
			local sy = h * 0.5

			love.graphics.setColor(theme.bg_focus)
			for i, v in ipairs(self.lines) do
				local y = 0.2 * (math.log(v))
				love.graphics.line(0, ty - sy * y, w, ty - sy * y)
			end

			for i = -9, 0 do
				local x = 300 * (math.log(2 ^ i))
				love.graphics.line(tx + x * sx, 0, tx + x * sx, h)
			end

			love.graphics.setColor(theme.ui_text)
			for i = 1, n - 1 do
				local x1 = 300 * (math.log(i / n))
				local x2 = 300 * (math.log((i + 1) / n))

				local y1 = 0.2 * (math.log(self.average[i]))
				local y2 = 0.2 * (math.log(self.average[i + 1]))

				love.graphics.line(tx + x1 * sx, ty - sy * y1, tx + x2 * sx, ty - sy * y2)
			end
		end
	else
		local scope = backend:getScope()
		if scope then
			local n = #scope

			local tx = 0 --w * 0.05
			local ty = h * 0.5
			local sx = 1 --w / n
			local sy = h * 0.5

			local n_max = math.min(n, math.floor(w / sx))

			local max = 0
			for i = 1, n_max do
				max = math.max(max, scope[i])
			end

			local threshold = 0.5 * max
			local x_first = 0
			local schmitt = true
			for i = 1, n_max do
				local trigger = false
				if schmitt then
					if scope[i] < -threshold then
						schmitt = false
					end
				else
					if scope[i] > threshold then
						schmitt = true

						trigger = true
					end
				end

				if trigger then
					if x_first == 0 then
						x_first = tx + i * sx
					end
				end
			end
			love.graphics.setColor(theme.ui_text)
			for i = 1, n - 1 do
				love.graphics.line(
					tx + i * sx - x_first,
					ty - sy * scope[i],
					tx + (i + 1) * sx - x_first,
					ty - sy * scope[i + 1]
				)
			end
		end
	end

	self.ui:draw()
end

return Scope
