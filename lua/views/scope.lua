local View = require("view")
local backend = require("backend")
local Ui = require("ui/ui")
local widgets = require("ui/widgets")
local Scope = View:derive("Scope")

-- TODO: make scope tracking better
--       zero crossing detection? frequency tracking?

function Scope:new(spectrum)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	local index = 1
	if spectrum then
		index = 2
	end

	new.ui = Ui:new(new)

	new.selector = widgets.Selector:new({ "scope", "spectrum" }, index)

	return new
end

function Scope:update()
	local w, h = self:getDimensions()

	self.ui:startFrame()
	self.ui.layout:col(200)
	self.ui:put(self.selector)
	self.ui:endFrame()
end

function Scope:draw()
	local w, h = self:getDimensions()

	love.graphics.setColor(theme.ui_text)

	if self.selector.index == 2 then
		local spectrum = backend:getSpectrum()
		if spectrum then
			local n = #spectrum

			local tx = w * 0.95
			local ty = h * 0.1
			local sx = (w * 0.9) / n
			local sy = h * 0.5

			for i = 1, n - 1 do
				local x1 = 300 * (math.log(i / n))
				local x2 = 300 * (math.log((i + 1) / n))

				local y1 = 0.2 * (math.log(spectrum[i]))
				local y2 = 0.2 * (math.log(spectrum[i + 1]))

				love.graphics.line(tx + x1 * sx, ty - sy * y1, tx + x2 * sx, ty - sy * y2)
			end
		end
	else
		local scope = backend:getScope()
		if scope then
			local n = #scope

			local tx = 0 --w * 0.05
			local ty = h * 0.5
			local sx = w / n
			local sy = h * 0.8

			for i = 1, n - 1 do
				love.graphics.line(tx + i * sx, ty - sy * scope[i], tx + (i + 1) * sx, ty - sy * scope[i + 1])
			end
		end
	end

	self.ui:draw()
end

return Scope
