local ui = require("ui")

local Slider = {}

function Slider:new(p)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.p = p

	return new
end

function Slider:stencil(x, y, w, h)
	love.graphics.rectangle("fill", x, y, w, h, ui.SLIDER_RADIUS)
end

function Slider:getPad(w)
	local pad = math.min(w * 0.4 - 64, ui.SLIDER_TEXT_PAD)
	if pad < ui.SLIDER_OFFSET then
		pad = ui.SLIDER_OFFSET
	end

	return pad
end

function Slider:draw(y, w, mode)
	local pad = Slider:getPad(w)
	local xs = pad
	local ys = y + 0.5 * (ui.GRID - ui.SLIDER_HEIGHT)
	local ws = (w - pad) - ui.SLIDER_OFFSET

	local v = self.p:getNormalized()

	love.graphics.stencil(function()
		self:stencil(xs, ys, ws, ui.SLIDER_HEIGHT)
	end, "increment", 1, true)
	love.graphics.setStencilTest("greater", 2)

	if mode == "hover" then
		love.graphics.setColor(theme.widget_hover)
	elseif mode == "press" then
		love.graphics.setColor(theme.widget_press)
	else
		love.graphics.setColor(theme.widget_bg)
	end
	love.graphics.rectangle("fill", xs, ys, ws, ui.SLIDER_HEIGHT)
	love.graphics.setColor(theme.widget)
	if self.p.centered then
		love.graphics.rectangle("fill", xs + ws * 0.5, ys, ws * (v - 0.5), ui.SLIDER_HEIGHT)
		-- love.graphics.rectangle("line", xs+ws*0.5, ys, ws*(v-0.5), ui.SLIDER_HEIGHT)
	else
		love.graphics.rectangle("fill", xs, ys, ws * v, ui.SLIDER_HEIGHT)
	end

	love.graphics.setStencilTest("greater", 1)

	love.graphics.setColor(theme.widget_line)
	love.graphics.rectangle("line", xs, ys, ws, ui.SLIDER_HEIGHT, ui.SLIDER_RADIUS)

	love.graphics.setColor(theme.ui_text)
	util.drawText(self.p.name, 0, y, xs, ui.GRID, "right")
	util.drawText(self.p:getDisplay(), xs, y - 1, ws, ui.GRID, "center")
end

function Slider:dragStart()
	self.pv = self.p:getNormalized()
end

function Slider:reset()
	self.p:reset()
end

function Slider:drag(w)
	local pad = self:getPad(w)
	local ws = (w - pad) - ui.SLIDER_OFFSET
	local scale = 0.7 / ws
	-- scale = 0.002

	local v = util.clamp(self.pv + scale * mouse.dx, 0, 1)
	self.p:setNormalized(v)
end

function Slider:getPosition(w)
	local pad = self:getPad(w)
	local xs = pad
	local ws = (w - pad) - ui.SLIDER_OFFSET

	return xs + ws * self.p:getNormalized()
end

return Slider
