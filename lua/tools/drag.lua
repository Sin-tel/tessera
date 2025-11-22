local tuning = require("tuning")
local util = require("util")

local drag = {}

drag.x_start = 0
drag.y_start = 0

function drag:mousepressed(canvas)
	local mx, my = canvas:getMouse()
	local closest, _ = canvas:find_closest_note(mx, my)
	self.note_origin = util.clone(closest)

	self.start_x, self.start_y = canvas.transform:inverse(mouse.x, mouse.y)
	self.prev_state = util.clone(selection.list)

	self.edit = false
end

function drag:mousedown(canvas)
	if not mouse.drag then
		return
	end
	local mx, my = canvas.transform:inverse(mouse.x, mouse.y)

	local x = mx - self.start_x
	local y = my - self.start_y

	if modifier_keys.shift then
		-- Snap time to grid
		local ix = self.note_origin.time
		x = (math.floor((ix + x) * 4 + 0.5) / 4) - ix
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
			v.pitch[j] = self.prev_state[i].pitch[j] + delta[j]
		end

		v.time = self.prev_state[i].time + x
	end
	-- canvas.transform:drag(px, py)
end

function drag:mousereleased(canvas)
	if mouse.drag then
		local c = command.noteUpdate.new(self.prev_state, selection.list)
		command.register(c)
		self.prev_state = nil
	end
end

function drag:draw(canvas) end

return drag
