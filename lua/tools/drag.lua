local time = require("time")
local tuning = require("tuning")

local drag = {}

drag.mode = "drag"

function drag:mousepressed(canvas)
	local mx, my = canvas:get_mouse()

	local closest, _ = canvas:find_closest_note(mx, my, 32)

	self.note_origin = selection.list[1]
	if closest then
		self.note_origin = util.clone(closest)
	end

	assert(self.note_origin)

	self.start_x, self.start_y = canvas.transform:inverse(mx, my)

	-- self.start_snap = tuning.snap(self.start_y)
	self.start_snap = self.note_origin

	self.edit = false
end

function drag:mousedown(canvas)
	local mx, my = canvas:get_mouse()
	local cx, cy = canvas.transform:inverse(mx, my)

	if not self.edit then
		if util.dist(self.start_x, self.start_y, mx, my) < mouse.DRAG_DIST then
			return
		else
			if drag.mode == "clone" then
				local notes = util.clone(selection.get_notes())
				for ch_index in pairs(notes) do
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

	local x = cx - self.start_x
	local y = cy - self.start_y

	if modifier_keys.shift then
		-- Constrain to one axis
		if math.abs(self.start_x - mx) < math.abs(self.start_y - my) then
			x = 0
		else
			y = 0
		end
	end

	local ix = self.note_origin.time
	x = time.snap(ix + x) - ix

	-- calculate relative offset
	local p_origin = tuning.snap_interval(self.note_origin.interval)
	local p = tuning.get_pitch(p_origin)
	local delta = tuning.snap(p + y)
	delta = tuning.sub(delta, p_origin)

	-- local f = tuning.snap(cy)
	-- local delta = tuning.sub(f, self.note_origin.interval)

	-- Update interval and time
	for i, v in ipairs(selection.list) do
		v.interval = tuning.add(self.prev_state[i].interval, delta)
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
