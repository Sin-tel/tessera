-- UI layout.
-- cargo run -- --test-run tests/lua/ui.lua

local Layout = require("ui/layout")

local h = dofile("tests/lua/lib/harness.lua")

local tests = {}

-- The total height decides how far a view can scroll.
tests["total height counts the last row of columns"] = function()
	local layout = Layout.new(100)
	layout:start(0, 0)
	local row_h = layout.h

	layout:row()
	assert(layout:total_height() == row_h, "one row")

	layout:col(50)
	layout:col(50)
	assert(layout:total_height() == 2 * row_h, "row of columns not counted: " .. layout:total_height())

	layout:new_row()
	layout:col(50)
	assert(layout:total_height() == 3 * row_h, "second row of columns: " .. layout:total_height())

	layout:row()
	assert(layout:total_height() == 4 * row_h, "row after columns: " .. layout:total_height())
end

return h.run(tests)
