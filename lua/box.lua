local ui = require("ui/ui")
local views = require("views")

local Box = {}
Box.__index = Box

function Box.new(x, y, w, h)
	local self = setmetatable({}, Box)

	self.x = x or 0
	self.y = y or 0
	self.w = w or width
	self.h = h or height

	self:set_view(views.Empty.new())

	return self
end

function Box.from_data()
	error("TODO")
end

function Box:to_data()
	local data = {}

	if self.children then
		data.a = self.children[1]:to_data()
		data.b = self.children[2]:to_data()
		data.vertical = self.vertical
		data.r = self.r
	else
		data.view = views.get_class_name(self.view)
		-- TODO: can also persist view data here (transform etc.)
	end
	return data
end

function Box:scissor()
	tessera.graphics.set_scissor(
		self.x + ui.BORDER_SIZE,
		self.y + ui.BORDER_SIZE,
		self.w - (2 * ui.BORDER_SIZE),
		self.h - (2 * ui.BORDER_SIZE)
	)
end

function Box:draw()
	if self.children then
		for _, v in ipairs(self.children) do
			v:draw()
		end
	else
		self:scissor()

		tessera.graphics.set_color(theme.background)
		tessera.graphics.rectangle("fill", self.x, self.y, self.w, self.h)

		tessera.graphics.push()
		tessera.graphics.translate(self.x, self.y)
		self.view:draw_full()
		tessera.graphics.pop()

		tessera.graphics.reset_scissor()

		tessera.graphics.set_color(theme.borders)
		tessera.graphics.rectangle(
			"line",
			self.x + ui.BORDER_SIZE,
			self.y + ui.BORDER_SIZE,
			self.w - (2 * ui.BORDER_SIZE),
			self.h - (2 * ui.BORDER_SIZE),
			ui.BORDER_RADIUS
		)
	end
end

function Box:get_divider()
	local mx = mouse.x - self.x
	local my = mouse.y - self.y
	if mx < 0 or my < 0 or mx > self.w or my > self.h then
		return false
	end
	if self.children then
		if self.vertical then
			if math.abs(self.r * self.w - mx) < ui.RESIZE_W then
				return self
			else
				local d1 = self.children[1]:get_divider()
				if d1 then
					return d1
				end
				local d2 = self.children[2]:get_divider()
				if d2 then
					return d2
				end
			end
		else
			if math.abs(self.r * self.h - my) < ui.RESIZE_W then
				return self
			else
				local d1 = self.children[1]:get_divider()
				if d1 then
					return d1
				end
				local d2 = self.children[2]:get_divider()
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
		self.x = x
		self.y = y
		self.w = w
		self.h = h
	end

	self.x = math.floor(self.x + 0.5)
	self.y = math.floor(self.y + 0.5)
	self.w = math.floor(self.w + 0.5)
	self.h = math.floor(self.h + 0.5)

	if self.children then
		if self.vertical then
			local w2 = self.r * self.w
			self.children[1]:recalc(self.x, self.y, w2, self.h)
			self.children[2]:recalc(self.x + w2, self.y, self.w - w2, self.h)
		else
			local h2 = self.r * self.h
			self.children[1]:recalc(self.x, self.y, self.w, h2)
			self.children[2]:recalc(self.x, self.y + h2, self.w, self.h - h2)
		end
	end

	if self.view then
		self.view:set_dimensions()
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
		self.r = (x - self.x) / self.w
	else
		self.r = (y - self.y) / self.h
	end

	if self.vertical then
		local bleft = self.children[1]:get_bound_left() - self.x + ui.MIN_SIZE
		local bright = self.children[2]:get_bound_right() - self.x - ui.MIN_SIZE
		self.r = math.min(math.max(self.r * self.w, bleft), bright) / self.w
	else
		local bup = self.children[1]:get_bound_up() - self.y + ui.MIN_SIZE
		local bdown = self.children[2]:get_bound_down() - self.y - ui.MIN_SIZE
		self.r = math.min(math.max(self.r * self.h, bup), bdown) / self.h
	end

	self:recalc()
end

function Box:split(r, vertical)
	if self.children then
		return false
	end
	self.children = {}
	self.vertical = vertical
	self.r = r
	if vertical then
		table.insert(self.children, Box.new())
		table.insert(self.children, Box.new())
	else
		table.insert(self.children, Box.new())
		table.insert(self.children, Box.new())
	end

	self:recalc()

	return self.children[1], self.children[2]
end

function Box:resize(x, y, w, h)
	self.x = x
	self.y = y
	self.w = w
	self.h = h

	if self.children then
		local w2 = self.r * self.w
		local h2 = self.r * self.h

		if self.vertical then
			self.r = math.min(math.max(w2, ui.MIN_SIZE), self.w - ui.MIN_SIZE) / self.w
			self.children[1]:resize(self.x, self.y, w2, self.h)
			self.children[2]:resize(self.x + w2, self.y, self.w - w2, self.h)
		else
			self.r = math.min(math.max(h2, ui.MIN_SIZE), self.h - ui.MIN_SIZE) / self.h
			self.children[1]:resize(self.x, self.y, self.w, h2)
			self.children[2]:resize(self.x, self.y + h2, self.w, self.h - h2)
		end

		self:set_split(self.x + w2, self.y + h2)
	end
end

function Box:update_view()
	if self.view then
		self.view:update()
	end
	if self.children then
		self.children[1]:update_view()
		self.children[2]:update_view()
	end
end

function Box:set_view(view)
	assert(not self.children)
	if not self.children then
		self.view = view
		view.box = self
	end
end

function Box:set_focus(focus)
	self.focus = focus
	if self.children then
		self.children[1]:set_focus(focus)
		self.children[2]:set_focus(focus)
	end
end

return Box
