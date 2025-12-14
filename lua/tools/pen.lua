local tuning = require("tuning")

-- TODO: duplicated in Roll
local DEFAULT_PRESSURE = 0.3

-- TODO: duplicated in note_input
-- TODO: re-use last edited velocity
local DEFAULT_VELOCITY = 0.5

-- used for click without dragging
local initial_length = (1 / 4)

local pen = {}

pen.ox = 0
pen.oy = 0

pen.pitch = tuning.new_note()

-- TODO: better rounding logic
-- TODO: query current local grid
local function get_pitch(canvas, my)
	local f = canvas.transform:pitch_inv(my)
	local chromatic = math.floor(f + 0.5)
	return tuning.from_midi(chromatic)
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

		self.t1 = canvas.transform:time_inv(self.ox)
		self.t2 = self.t1 + initial_length

		self.pitch = get_pitch(canvas, my)

		self.active = true

		-- trigger note_on
		self.token = tessera.audio.get_token()
		ui_channels[selection.ch_index]:send_event({
			name = "note_on",
			token = self.token,
			pitch = self.pitch,
			vel = DEFAULT_VELOCITY,
			offset = 0,
		})
	end
end

function pen:mousedown(canvas)
	if not self.active then
		return
	end

	if mouse.drag then
		local mx, my = canvas:get_mouse()
		self.t2 = canvas.transform:time_inv(mx)
		local new_pitch = get_pitch(canvas, my)

		-- trigger a new note if pitch dragged
		if not tuning.eq(new_pitch, self.pitch) then
			self.pitch = new_pitch
			assert(self.token)
			ui_channels[selection.ch_index]:send_event({
				name = "note_off",
				token = self.token,
			})

			self.token = tessera.audio.get_token()
			ui_channels[selection.ch_index]:send_event({
				name = "note_on",
				token = self.token,
				pitch = self.pitch,
				vel = DEFAULT_VELOCITY,
				offset = 0,
			})
		end
	end
end

function pen:update(canvas)
	local _, my = canvas:get_mouse()
	self.pitch = get_pitch(canvas, my)
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
		pitch = self.pitch,
		time = t1,
		vel = vel,
		verts = { { 0.0, offset, pressure }, { t2 - t1, offset, pressure } },
	}

	local notes = {}
	notes[ch_index] = { note }

	local c = command.NoteAdd.new(notes)
	command.run_and_register(c)

	assert(self.token)
	ui_channels[selection.ch_index]:send_event({
		name = "note_off",
		token = self.token,
	})
	self.token = nil
end

function pen:draw(canvas)
	local mx, my = canvas:get_mouse()

	local p = tuning.get_pitch(self.pitch)
	local y = canvas.transform:pitch(p)

	local lx, ly = mx, my
	if self.active then
		ly = y
		lx = math.min(self.ox, mx)
	end

	-- draw note label
	tessera.graphics.set_color(theme.text_tip)
	local note_name = tuning.get_name(self.pitch)
	tessera.graphics.text(note_name, lx + 5, ly - 20)

	-- draw note preview
	if self.active then
		local ch = project.channels[selection.ch_index]
		local c = tessera.graphics.get_color_hsv(ch.hue / 360, 0.75, 0.80)
		tessera.graphics.set_color(c)

		tessera.graphics.set_line_width(2.0)
		tessera.graphics.line(self.ox, y, mx, y)
		tessera.graphics.circle("fill", lx, y, 3)
	end
end

return pen
