Slider = {}

UI_GRID = 28
SLIDER_HEIGHT = 19
SLIDER_RADIUS = 6
SLIDER_TEXT_PAD = 150
SLIDER_OFFSET = 4

function Slider:new(name)
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.v = math.random()
	new.name = name or "Value"

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

function Slider:draw(y, w, hover)

	local pad = Slider:getPad(w)
	local xs = pad
	local ys = y + 0.5*(UI_GRID - SLIDER_HEIGHT)
	local ws = (w-pad)-SLIDER_OFFSET

	love.graphics.stencil( function() self:stencil(xs, ys, ws, SLIDER_HEIGHT) end, "increment", 1 , true )
	love.graphics.setStencilTest("greater", 2)

	if hover then
		love.graphics.setColor(Theme.slider_hover)
	else
		love.graphics.setColor(Theme.slider_bg)
	end
	love.graphics.rectangle("fill", xs, ys, ws, SLIDER_HEIGHT)
	love.graphics.setColor(Theme.slider)
	love.graphics.rectangle("fill", xs, ys, ws*self.v, SLIDER_HEIGHT)

	love.graphics.setStencilTest("greater", 1)

	love.graphics.setColor(Theme.slider_line)
	love.graphics.rectangle("line", xs, ys, ws, SLIDER_HEIGHT, SLIDER_RADIUS)

	love.graphics.setColor(Theme.ui_text)
	drawText(self.name, 0, y, xs, UI_GRID, "right")
	drawText(string.format("%0.3f", self.v), xs, y-1, ws, UI_GRID, "center")
end

function Slider:dragStart()
	self.pv = self.v
end

function Slider:drag(w)
	-- local pad = self:getPad(w)
	-- local ws = (w-pad)-SLIDER_OFFSET
	-- local scale = 1/ws
	scale = 0.002

	if love.keyboard.isDown("lshift") then
		self.v = clamp(self.pv + 0.1*scale*Mouse.dx, 0, 1)
	else
		self.v = clamp(self.pv + scale*Mouse.dx, 0, 1)		
	end
end

function Slider:getPosition(w)
	local pad = self:getPad(w)
	local xs = pad
	local ws = (w-pad)-SLIDER_OFFSET

	return xs + ws*self.v
end