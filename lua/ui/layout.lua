local Ui = require("ui/ui")

local Layout = {}

-- TODO: automatic layout for columns
-- TODO: max layout for columns (fill rest of columns)
-- TODO: make width and height seperate from row_width for easier auto layouts
function Layout:new(w, h)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	self.x = 0
	self.y = 0

	self.w = w or 16
	self.h = Ui.ROW_HEIGHT

	self.ok = false

	self.pad = Ui.DEFAULT_PAD

	self.column_mode = false

	return new
end

function Layout:start(x, y, w, h)
	self.x = x or 0
	self.y = y or 0

	self.ok = false
	self.column_mode = false

	self.start_x = self.x
	self.start_y = self.y

	self.next_x = 0
	self.next_y = 0
	self.next_w = 0
	self.next_h = 0
end

function Layout:newRow()
	self.y = self.y + self.h
	self.column_mode = false
	self.x = self.start_x
end

function Layout:row(w, h)
	self.w = w or self.w
	self.h = h or self.h

	-- if we just did columns we need to start a new row
	if self.column_mode then
		self:newRow()
	end

	local x, y = self.start_x, self.y

	-- self.x = self.start_x
	self.y = self.y + self.h

	assert(self.w ~= nil)
	assert(self.h ~= nil)

	self.next_x = x + self.pad
	self.next_y = y + self.pad
	self.next_w = self.w - 2 * self.pad
	self.next_h = self.h - 2 * self.pad

	self.column_mode = false
	self.ok = true
end

function Layout:col(w, h)
	self.w = w or self.w
	self.h = h or self.h

	local x, y = self.x, self.y

	self.x = self.x + self.w

	self.next_x = x + self.pad
	self.next_y = y + self.pad
	self.next_w = self.w - 2 * self.pad
	self.next_h = self.h - 2 * self.pad

	self.column_mode = true
	self.ok = true
end

function Layout:totalHeight()
	return self.y - self.start_y
end

function Layout:get()
	assert(self.ok)
	self.ok = false
	return self.next_x, self.next_y, self.next_w, self.next_h
end

return Layout
