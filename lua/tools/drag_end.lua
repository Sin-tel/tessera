local util = require("util")

local drag_end = {}

drag_end.note_origin = nil
drag_end.x_start = 0
drag_end.y_start = 0
drag_end.selection = {}

local min_t = (1 / 16)

function drag_end:set_note_origin(note_origin)
	self.note_origin = util.clone(note_origin)
end

function drag_end:mousepressed(canvas)
	self.start_x, self.start_y = canvas.transform:inverse(mouse.x, mouse.y)
	self.selection = util.clone(selection.list)
end

function drag_end:mousedown(canvas)
	local mx, _ = canvas.transform:inverse(mouse.x, mouse.y)
	local x = mx - self.start_x

	-- if not modifier_keys.shift then
	-- 	-- Snap time to grid
	-- 	x = math.floor(x * 4 + 0.5) / 4
	-- end

	-- Update pitch and time of selected notes
	for i, v in ipairs(selection.list) do
		local n = #v.verts
		local new_t = self.selection[i].verts[n][1] + x
		v.verts[n][1] = math.max(min_t, new_t)
	end
end

function drag_end:mousereleased(canvas) end

function drag_end:draw(canvas) end

return drag_end
