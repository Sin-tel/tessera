local backend = require("backend")
local ui = require("ui/ui")

local workspace = {}

local Box = {}

function Box:new(x, y, w, h)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.x = x or 0
	new.y = y or 0
	new.w = w or width
	new.h = h or height

	return new
end

function Box:stencil()
	love.graphics.rectangle(
		"fill",
		self.x + ui.BORDER_SIZE,
		self.y + ui.BORDER_SIZE,
		self.w - (2 * ui.BORDER_SIZE),
		self.h - (2 * ui.BORDER_SIZE),
		ui.BORDER_RADIUS
	)
end

function Box:forAll(f)
	f(self)
	if self.children then
		self.children[1]:forAll(f)
		self.children[2]:forAll(f)
	end
end

function Box:draw()
	if self.children then
		for _, v in ipairs(self.children) do
			v:draw()
		end
	else
		love.graphics.stencil(function()
			self:stencil()
		end, "replace", 2, false)
		love.graphics.setStencilTest("greater", 1)

		love.graphics.setColor(theme.background)
		love.graphics.rectangle("fill", self.x, self.y, self.w, self.h)

		love.graphics.push()
		love.graphics.translate(self.x, self.y)
		self.view:drawFull()
		love.graphics.pop()

		love.graphics.setStencilTest()
		love.graphics.setColor(theme.borders)
		love.graphics.rectangle(
			"line",
			self.x + ui.BORDER_SIZE,
			self.y + ui.BORDER_SIZE,
			self.w - (2 * ui.BORDER_SIZE),
			self.h - (2 * ui.BORDER_SIZE),
			ui.BORDER_RADIUS
		)
	end
end

function Box:getDivider()
	local mx = mouse.x - self.x
	local my = mouse.y - self.y
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			if math.abs(self.r - mx) < ui.RESIZE_W then
				return self
			else
				local d1 = self.children[1]:getDivider()
				if d1 then
					return d1
				end
				local d2 = self.children[2]:getDivider()
				if d2 then
					return d2
				end
			end
		else
			if math.abs(self.r - my) < ui.RESIZE_W then
				return self
			else
				local d1 = self.children[1]:getDivider()
				if d1 then
					return d1
				end
				local d2 = self.children[2]:getDivider()
				if d2 then
					return d2
				end
			end
		end
	end
	return false
end

function Box:get()
	local mx = mouse.x - self.x
	local my = mouse.y - self.y
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			local d1 = self.children[1]:get()
			if d1 then
				return d1
			end
			local d2 = self.children[2]:get()
			if d2 then
				return d2
			end
		else
			local d1 = self.children[1]:get()
			if d1 then
				return d1
			end
			local d2 = self.children[2]:get()
			if d2 then
				return d2
			end
		end
	end
	return self
end

function Box:recalc(x, y, w, h)
	if x then
		if self.children then
			if self.vertical then
				self.r = self.r + (self.x - x)
			else
				self.r = self.r + (self.y - y)
			end
		end
		self.x = x
		self.y = y
		self.w = w
		self.h = h
	end

	self.x = math.floor(self.x)
	self.y = math.floor(self.y)
	self.w = math.floor(self.w)
	self.h = math.floor(self.h)

	if self.children then
		if self.vertical then
			self.children[1]:recalc(self.x, self.y, self.r, self.h)
			self.children[2]:recalc(self.x + self.r, self.y, self.w - self.r, self.h)
		else
			self.children[1]:recalc(self.x, self.y, self.w, self.r)
			self.children[2]:recalc(self.x, self.y + self.r, self.w, self.h - self.r)
		end
	end
end

function Box:get_bound_left()
	if self.children then
		return math.max(self.children[1]:get_bound_left(), self.children[2]:get_bound_left())
	else
		return self.x
	end
end

function Box:get_bound_right()
	if self.children then
		return math.min(self.children[1]:get_bound_right(), self.children[2]:get_bound_right())
	else
		return self.x + self.w
	end
end

function Box:get_bound_up()
	if self.children then
		return math.max(self.children[1]:get_bound_up(), self.children[2]:get_bound_up())
	else
		return self.y
	end
end

function Box:get_bound_down()
	if self.children then
		return math.min(self.children[1]:get_bound_down(), self.children[2]:get_bound_down())
	else
		return self.y + self.h
	end
end

function Box:set_split(x, y)
	if not self.children then
		error("Box has no div")
	end
	if self.vertical then
		self.r = (x - self.x)
	else
		self.r = (y - self.y)
	end

	if self.vertical then
		local bleft = self.children[1]:get_bound_left() - self.x + ui.MIN_SIZE
		local bright = self.children[2]:get_bound_right() - self.x - ui.MIN_SIZE
		self.r = math.min(math.max(self.r, bleft), bright)
	else
		local bup = self.children[1]:get_bound_up() - self.y + ui.MIN_SIZE
		local bdown = self.children[2]:get_bound_down() - self.y - ui.MIN_SIZE
		self.r = math.min(math.max(self.r, bup), bdown)
	end

	self:recalc()
end

function Box:split(r, vertical)
	if self.children then
		return false
	end
	self.children = {}
	self.vertical = vertical
	if vertical then
		self.r = self.w * r
		table.insert(self.children, Box:new())
		table.insert(self.children, Box:new())
	else
		self.r = self.h * r
		table.insert(self.children, Box:new())
		table.insert(self.children, Box:new())
	end

	self:recalc()

	return self.children[1], self.children[2]
end

function Box:resize(x, y, w, h)
	if self.children then
		if self.vertical then
			self.r = w * self.r / self.w
		else
			self.r = h * self.r / self.h
		end
	end
	self.x = x
	self.y = y
	self.w = w
	self.h = h

	if self.children then
		if self.vertical then
			self.r = math.min(math.max(self.r, ui.MIN_SIZE), self.w - ui.MIN_SIZE)
		else
			self.r = math.min(math.max(self.r, ui.MIN_SIZE), self.h - ui.MIN_SIZE)
		end
		if self.vertical then
			self.children[1]:resize(self.x, self.y, self.r, self.h)
			self.children[2]:resize(self.x + self.r, self.y, self.w - self.r, self.h)
		else
			self.children[1]:resize(self.x, self.y, self.w, self.r)
			self.children[2]:resize(self.x, self.y + self.r, self.w, self.h - self.r)
		end

		self:set_split(self.x + self.r, self.y + self.r)
	end
end

function Box:setView(view)
	if not self.children then
		self.view = view
		view.box = self
	end
end

function workspace:load()
	self.w = width
	self.h = height
	self.box = Box:new(0, ui.RIBBON_HEIGHT, width, height - ui.RIBBON_HEIGHT)

	self.cpu_load = 0
	self.meter = { l = -math.huge, r = -math.huge }
end

function workspace:resize(w, h)
	self.w = w
	self.h = h

	self.box:resize(0, ui.RIBBON_HEIGHT, w, h - ui.RIBBON_HEIGHT)
	self.box:resize(0, ui.RIBBON_HEIGHT, w, h - ui.RIBBON_HEIGHT) -- second time to satisfy constraints properly
end

function workspace:draw()
	local ll = util.clamp(self.cpu_load, 0, 1)
	local hl_col = theme.cpu_meter
	if self.cpu_load > 1.0 then
		hl_col = theme.warning
	end

	local w1 = 64
	local h1 = 16
	local y1 = 0.5 * (ui.RIBBON_HEIGHT - h1)
	local x1 = self.w - 64 - y1

	love.graphics.setColor(theme.widget_bg)
	love.graphics.rectangle("fill", x1, y1, w1, h1, 2)
	love.graphics.setColor(hl_col)
	love.graphics.rectangle("fill", x1, y1, w1 * ll, h1)
	love.graphics.setColor(theme.line)
	love.graphics.rectangle("line", x1, y1, w1, h1, 2)
	love.graphics.setColor(theme.ui_text)
	if backend:running() then
		util.drawText(string.format("%d %%", 100 * self.cpu_load), x1, 0, w1, ui.RIBBON_HEIGHT, "center")
	else
		util.drawText("offline", x1, 0, w1, ui.RIBBON_HEIGHT, "center")
	end
	util.drawText("CPU: ", x1 - w1, 0, w1, ui.RIBBON_HEIGHT, "right")

	w1 = 96
	h1 = 16
	y1 = 0.5 * (ui.RIBBON_HEIGHT - h1)
	x1 = self.w - 224 - y1

	local ml = util.clamp((self.meter.l + 80) / 80, 0, 1)
	local mr = util.clamp((self.meter.r + 80) / 80, 0, 1)

	love.graphics.setColor(theme.widget_bg)
	love.graphics.rectangle("fill", x1, y1, w1, h1, 2)
	love.graphics.setColor(ml < 1.0 and theme.meter or theme.meter_clip)
	love.graphics.rectangle("fill", x1, y1, w1 * ml, 0.5 * h1 - 1)
	love.graphics.setColor(mr < 1.0 and theme.meter or theme.meter_clip)
	love.graphics.rectangle("fill", x1, y1 + 0.5 * h1, w1 * mr, 0.5 * h1)
	love.graphics.setColor(theme.line)
	love.graphics.rectangle("line", x1, y1, w1, h1, 2)
	love.graphics.setColor(theme.ui_text)

	self.box:draw()
end

function workspace:update()
	if self.dragDiv and mouse.drag then
		self.dragDiv:set_split(mouse.x, mouse.y)
	end
	-- update
	self.box:forAll(function(b)
		if b.view then
			b.view:update()
		end
	end)

	local div = self.dragDiv
	if not mouse.isDown then
		div = div or self.box:getDivider()
		self.box:forAll(function(b)
			b.focus = false
		end)
		self.focus = nil
	end
	if div then
		if div.vertical then
			mouse:setCursor("v")
		else
			mouse:setCursor("h")
		end
	else
		if not mouse.isDown then
			local b = self.box:get()
			if b then
				b.focus = true
				self.focus = b.view
			end
		end
	end
end

function workspace:mousepressed()
	local div = false
	if mouse.button == 1 then
		div = self.box:getDivider(mouse.x, mouse.y)
		if div then
			self.dragDiv = div
		end
	end

	if not div and self.focus then
		self.focus:mousepressed()
	end
end

function workspace:mousereleased()
	self.dragDiv = nil
	if self.focus then
		self.focus:mousereleased()
	end
end

function workspace:keypressed(key, mod)
	if self.focus then
		return self.focus:keypressed(key, mod)
	end
end

return workspace
