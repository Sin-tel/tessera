local util = require("util")
local tuning = require("tuning")

local drag = {}

drag.note_origin = nil
drag.x_start = 0
drag.y_start = 0
drag.selection = {}

function drag:set_note_origin(note_origin)
	self.note_origin = util.clone(note_origin)
end

function drag:mousepressed(canvas)
	self.start_x, self.start_y = canvas.transform:inverse(mouse.x, mouse.y)
	self.selection = util.clone(selection.list)
end

function drag:mousedown(canvas)
	local mx, my = canvas.transform:inverse(mouse.x, mouse.y)
	local x = mx - self.start_x
	local y = my - self.start_y

	if not modifier_keys.shift then
		-- Snap time to grid
		x = math.floor(x * 4 + 0.5) / 4
	end
	-- Get pitch location in local frame
	local n = tuning.getDiatonicIndex(self.note_origin.pitch)

	-- Calculate base pitch offset
	local steps = math.floor(y * (7 / 12) + 0.5)
	local p_origin = tuning.fromDiatonic(n)
	local delta = tuning.fromDiatonic(n + steps)
	for i, _ in ipairs(delta) do
		delta[i] = delta[i] - p_origin[i]
	end

	-- Update pitch and time of selected notes
	for i, v in ipairs(selection.list) do
		for j, _ in ipairs(delta) do
			v.pitch[j] = self.selection[i].pitch[j] + delta[j]
		end

		v.time = self.selection[i].time + x
	end
	-- canvas.transform:drag(px, py)
end

function drag:mousereleased(canvas) end

function drag:draw(canvas) end

return drag
