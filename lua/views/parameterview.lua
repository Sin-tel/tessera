local View = require("view")
local ui = require("ui")

local parameterView = View:derive("Parameters")

function parameterView:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.groups = {}

	new.select = nil
	new.action = nil

	new.scroll = 0
	new.scroll_ = 0

	return new
end

function parameterView:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.scroll = self.scroll + (self.scroll_ - self.scroll) * 0.5
	self.scroll = util.clamp(self.scroll, 0, self:getMaxScroll())
	self.scroll_ = util.clamp(self.scroll_, 0, self:getMaxScroll())

	if math.abs(self.scroll - self.scroll_) < 2 then
		self.scroll = self.scroll_
	end

	if self.box.focus then
		if self.action == "slider" then
			if mouse.drag then
				self.select:drag(w)
			end
		else
			local index = math.floor((my + self.scroll) / ui.GRID)
			local y = 0
			self.select = nil
			for _, v in ipairs(self.groups) do
				if not v.collapse then
					local get = v.sliders[index - y]
					if get and mx > get:getPad(w) - 2 then
						self.select = get
						self.select_i = index
						break
					end
				end
				y = y + v:getLength()
			end
		end
	else
		self.select = nil
	end

	if selection.channel then
		self.groups = selection.channel.parameterGroups
	end
end

function parameterView:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if mouse.button == 1 then
		if self.select then
			self.select:dragStart()

			love.mouse.setRelativeMode(true)
			self.action = "slider"
		else
			local index = math.floor((my + self.scroll) / ui.GRID)

			for _, v in ipairs(self.groups) do
				if index == 0 then
					v.collapse = not v.collapse
					break
				end
				index = index - v:getLength()
			end
		end
	elseif mouse.button == 2 then
		if self.select then
			self.select:reset()
		end
	end
end

function parameterView:mousereleased()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if self.action == "slider" then
		local s = self.select:getPosition(w)
		love.mouse.setRelativeMode(false)
		if mouse.drag then
			love.mouse.setPosition(
				self.box.x + s,
				self.box.y + self.select_i * ui.GRID + ui.HEADER + 0.5 * ui.GRID - self.scroll
			)
		end
	end
	self.action = nil
end

function parameterView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	local y = -self.scroll / ui.GRID
	for _, group in ipairs(self.groups) do
		group:draw(y, w, self.select)
		y = y + group:getLength()
	end
end

function parameterView:wheelmoved(y)
	self.scroll_ = math.floor(self.scroll - y * 1.5 * ui.GRID)
end

function parameterView:getMaxScroll()
	local w, h = self:getDimensions()

	local l = 0
	for _, group in ipairs(self.groups) do
		l = l + group:getLength()
	end
	return math.max(0, (l * ui.GRID) - h)
end

return parameterView
