Slider = {}

UI_GRID = 28
SLIDER_HEIGHT = 18
SLIDER_RADIUS = 6
SLIDER_TEXT_PAD = 150
SLIDER_OFFSET = 8

function Slider:new(p)
	local new = {}
	setmetatable(new,self)
	self.__index = self
	
	new.p = p

	return new	
end

function Slider:stencil(x,y,w,h)
	love.graphics.rectangle("fill", x,y,w,h, SLIDER_RADIUS)
end

function Slider:getPad(w)

	local pad = math.min(w*0.4 - 64, SLIDER_TEXT_PAD)
	if pad < SLIDER_OFFSET then
		pad = SLIDER_OFFSET
	end

	return pad
end

function Slider:draw(y, w, mode)

	local pad = Slider:getPad(w)
	local xs = pad
	local ys = y + 0.5*(UI_GRID - SLIDER_HEIGHT)
	local ws = (w-pad)-SLIDER_OFFSET

	local v = self.p:getNormalized()

	love.graphics.stencil( function() self:stencil(xs, ys, ws, SLIDER_HEIGHT) end, "increment", 1 , true )
	love.graphics.setStencilTest("greater", 2)

	if mode == "hover" then
		love.graphics.setColor(Theme.widget_hover)
	elseif mode == "press" then
		love.graphics.setColor(Theme.widget_press)
	else
		love.graphics.setColor(Theme.widget_bg)
	end
	love.graphics.rectangle("fill", xs, ys, ws, SLIDER_HEIGHT)
	love.graphics.setColor(Theme.widget)
	if self.p.centered then
		love.graphics.rectangle("fill", xs+ws*0.5, ys, ws*(v-0.5), SLIDER_HEIGHT)
		-- love.graphics.rectangle("line", xs+ws*0.5, ys, ws*(v-0.5), SLIDER_HEIGHT)
	else
		love.graphics.rectangle("fill", xs, ys, ws*v, SLIDER_HEIGHT)
	end

	love.graphics.setStencilTest("greater", 1)

	love.graphics.setColor(Theme.widget_line)
	love.graphics.rectangle("line", xs, ys, ws, SLIDER_HEIGHT, SLIDER_RADIUS)

	love.graphics.setColor(Theme.ui_text)
	drawText(self.p.name, 0, y, xs, UI_GRID, "right")
	drawText(self.p:getDisplay(), xs, y-1, ws, UI_GRID, "center")
end

function Slider:dragStart()
	self.pv = self.p:getNormalized()
end

function Slider:reset()
	self.p:reset()
end

function Slider:drag(w)
	local pad = self:getPad(w)
	local ws = (w-pad)-SLIDER_OFFSET
	local scale = 0.7/ws
	-- scale = 0.002

	local v = clamp(self.pv + scale*Mouse.dx, 0, 1)		
	self.p:setNormalized(v)
end

function Slider:getPosition(w)
	local pad = self:getPad(w)
	local xs = pad
	local ws = (w-pad)-SLIDER_OFFSET

	return xs + ws*self.p:getNormalized()
end