local Ui = require("ui/ui")

local Layout = {}
Layout.__index = Layout

-- TODO: automatic layout for columns
-- TODO: max layout for columns (fill rest of columns)
-- TODO: make width and height seperate from row_width for easier auto layouts

function Layout.new(w, h)
	local self = setmetatable({}, Layout)

	self.x = 0
	self.y = 0

	self.w = w or 16
	self.h = Ui.ROW_HEIGHT

	self.ok = false

	self.pad = Ui.PAD

	self.column_mode = false

	return self
end

function Layout:padding(pad)
	self.pad = pad or Ui.PAD
end

function Layout:start(x, y)
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

	self.row_y = 0
	self.row_h = 0
end

function Layout:new_row()
	self.y = self.y + self.h
	self.column_mode = false
	self.x = self.start_x
end

function Layout:row(w, h)
	w = w or self.w
	h = h or self.h

	-- if we just did columns we need to start a new row
	if self.column_mode then
		self:new_row()
	end

	local x, y = self.start_x, self.y

	self.y = self.y + self.h

	self.next_x = x + self.pad
	self.next_y = y + self.pad
	self.next_w = w - 2 * self.pad
	self.next_h = h - 2 * self.pad

	self.row_y = y
	self.row_h = h

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

	self.row_y = y
	self.row_h = self.h

	self.column_mode = true
	self.ok = true
end

function Layout:total_height()
	return self.y - self.start_y
end

function Layout:get()
	-- assert(self.ok)
	self.ok = false
	return self.next_x, self.next_y, self.next_w, self.next_h
end

return Layout
