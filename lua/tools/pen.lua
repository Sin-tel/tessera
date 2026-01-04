local time = require("time")
local tuning = require("tuning")

-- TODO: duplicated in Roll
local DEFAULT_PRESSURE = 0.0

-- TODO: duplicated in note_input
-- TODO: re-use last edited velocity
local DEFAULT_VELOCITY = 0.5

-- TODO: re-use last length

local pen = {}

pen.ox = 0
pen.oy = 0

pen.interval = tuning.new_interval()

local function get_interval(canvas, my)
	local f = canvas.transform:pitch_inv(my) - 60
	return tuning.snap(f)
end

function pen:mousepressed(canvas)
	local ch_index = selection.ch_index
	if not ch_index then
		return
	end

	local channel = project.channels[ch_index]
	if channel.lock or not channel.visible then
		return
	end

	if selection.ch_index then
		local mx, my = canvas:get_mouse()

		self.ox = mx
		self.oy = my

		self.min_length = math.max(time.snap_length(), 1 / 64)

		self.t1 = time.snap(canvas.transform:time_inv(self.ox))
		self.t2 = self.t1 + math.max(time.snap_length(), 1 / 16)

		self.interval = get_interval(canvas, my)

		self.active = true

		-- trigger note_on
		if project.settings.preview_notes then
			self.token = tessera.audio.get_token()
			ui_channels[selection.ch_index]:send_event({
				name = "note_on",
				token = self.token,
				interval = self.interval,
				vel = DEFAULT_VELOCITY,
				offset = 0,
			})
		end
	end
end

function pen:mousedown(canvas)
	if not self.active then
		return
	end

	if mouse.drag then
		local mx, my = canvas:get_mouse()

		self.t2 = time.snap(canvas.transform:time_inv(mx))
		if math.abs(self.t1 - self.t2) < 0.01 then
			self.t2 = self.t1 + self.min_length
		end

		local new_interval = get_interval(canvas, my)

		if project.settings.preview_notes then
			-- trigger a new note if dragged
			if not tuning.eq(new_interval, self.interval) then
				if project.settings.preview_notes then
					self.interval = new_interval
					assert(self.token)
					ui_channels[selection.ch_index]:send_event({
						name = "note_off",
						token = self.token,
					})

					self.token = tessera.audio.get_token()
					ui_channels[selection.ch_index]:send_event({
						name = "note_on",
						token = self.token,
						interval = self.interval,
						vel = DEFAULT_VELOCITY,
						offset = 0,
					})
				end
			end
		else
			self.interval = new_interval
		end
	end
end

function pen:update(canvas)
	local _, my = canvas:get_mouse()
	if not self.active then
		-- update for preview label
		self.interval = get_interval(canvas, my)
	end
end

function pen:mousereleased(canvas)
	if not self.active then
		return
	end
	self.active = false

	local ch_index = selection.ch_index

	local offset = 0
	local vel = DEFAULT_VELOCITY
	local pressure = DEFAULT_PRESSURE

	local t1 = math.min(self.t1, self.t2)
	local t2 = math.max(self.t1, self.t2)

	local note = {
		interval = self.interval,
		time = t1,
		vel = vel,
		verts = { { 0.0, offset, pressure }, { t2 - t1, offset, pressure } },
	}

	local notes = {}
	notes[ch_index] = { note }

	local c = command.NoteAdd.new(notes)
	command.run_and_register(c)

	selection.add({ [note] = true })

	if project.settings.preview_notes then
		assert(self.token)
		ui_channels[selection.ch_index]:send_event({
			name = "note_off",
			token = self.token,
		})
		self.token = nil
	end
end

function pen:draw(canvas)
	local mx, my = canvas:get_mouse()

	local p = tuning.get_pitch(self.interval)
	local y = canvas.transform:pitch(p)

	local x1, y1 = mx, my
	if self.active then
		y1 = y
		x1 = canvas.transform:time(math.min(self.t1, self.t2))
	end

	-- draw note label
	tessera.graphics.set_color(theme.text_tip)
	local note_name = tuning.get_name(self.interval)
	tessera.graphics.text(note_name, x1 + 5, y1 - 20)

	-- draw note preview
	if self.active then
		local ch = project.channels[selection.ch_index]
		local c = tessera.graphics.get_color_hsv(ch.hue / 360, 0.75, 0.80)
		tessera.graphics.set_color(c)

		local x2 = canvas.transform:time(math.max(self.t1, self.t2))
		tessera.graphics.set_line_width(2.0)
		tessera.graphics.line(x1, y, x2, y)
		tessera.graphics.circle("fill", x1, y, 3)
	end
end

return pen
