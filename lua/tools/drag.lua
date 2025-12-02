local tuning = require("tuning")
local util = require("util")

local drag = {}

drag.mode = "drag"

function drag:mousepressed(canvas)
	self.ix, self.iy = mouse.x, mouse.y

	local mx, my = canvas:get_mouse()

	local closest, _ = canvas:find_closest_note(mx, my)
	self.note_origin = util.clone(closest)

	self.start_x, self.start_y = canvas.transform:inverse(mouse.x, mouse.y)

	self.edit = false
end

function drag:mousedown(canvas)
	if not self.edit then
		if util.dist(self.ix, self.iy, mouse.x, mouse.y) < mouse.DRAG_DIST then
			return
		else
			if drag.mode == "clone" then
				local notes = util.clone(selection.get_notes())
				for ch_index in ipairs(notes) do
					for _, note in ipairs(notes[ch_index]) do
						table.insert(project.channels[ch_index].notes, note)
					end
				end
				selection.set_from_notes(notes)
			end

			self.prev_state = util.clone(selection.list)
			self.edit = true
		end
	end

	local mx, my = canvas.transform:inverse(mouse.x, mouse.y)
	local x = mx - self.start_x
	local y = my - self.start_y

	if modifier_keys.shift then
		-- Constrain to one axis
		if math.abs(self.ix - mouse.x) < math.abs(self.iy - mouse.y) then
			x = 0
		else
			y = 0
		end
	end

	if modifier_keys.ctrl then
		-- Snap time to grid
		local ix = self.note_origin.time
		x = (math.floor((ix + x) * 4 + 0.5) / 4) - ix
	end

	-- Get pitch location in local frame
	local n = tuning.get_diatonic_index(self.note_origin.pitch)

	-- Calculate base pitch offset
	local steps = math.floor(y * (7 / 12) + 0.5)
	local p_origin = tuning.from_diatonic(n)
	local delta = tuning.from_diatonic(n + steps)
	for i, _ in ipairs(delta) do
		delta[i] = delta[i] - p_origin[i]
	end

	-- Update pitch and time
	for i, v in ipairs(selection.list) do
		for j, _ in ipairs(delta) do
			v.pitch[j] = self.prev_state[i].pitch[j] + delta[j]
		end

		v.time = self.prev_state[i].time + x
	end
end

function drag:mousereleased(canvas)
	if self.edit then
		if self.mode == "clone" or self.mode == "paste" then
			local notes = selection.get_notes()
			local c = command.NoteAdd.new(notes)
			command.register(c)
		else
			local c = command.NoteUpdate.new(self.prev_state, selection.list)
			command.register(c)
			self.prev_state = nil
		end

		self.mode = "drag"
		return true
	end

	self.mode = "drag"
end

function drag:draw(canvas) end

return drag
