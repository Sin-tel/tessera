local engine = require("engine")

local Roll = {}
Roll.__index = Roll

local DEFAULT_PRESSURE = 0.0

function Roll.new(ch_index)
	local self = setmetatable({}, Roll)

	self.ch_index = ch_index

	self.n_index = 1
	self.note_table = {}
	self.control_table = {}
	self.voices = {}

	self.active_notes = {}
	self.recorded_notes = {}

	return self
end

local function sort_time(a, b)
	return a.time < b.time
end

function Roll:start(chase)
	self.note_table = {}
	self.control_table = {}
	self.voices = {}

	for _, v in ipairs(project.channels[self.ch_index].notes) do
		assert(v.verts[1][1] == 0)
		table.insert(self.note_table, v)
	end
	table.sort(self.note_table, sort_time)

	-- dump all control messages in one table
	-- TODO: don't know if it's good to have two seperate tables
	for k, c in pairs(project.channels[self.ch_index].control) do
		for _, v in ipairs(c) do
			table.insert(self.control_table, { name = k, value = v.value, time = v.time })
		end
	end
	table.sort(self.control_table, sort_time)

	self:seek(chase)
end

function Roll:seek(chase)
	-- seek
	self.n_index = 1
	self.c_index = 1

	-- skip notes already played
	while self.note_table[self.n_index] do
		local note = self.note_table[self.n_index]
		local end_time = 0
		if chase then
			end_time = note.verts[#note.verts][1]
		end
		if note.time + end_time >= engine.time then
			break
		end
		self.n_index = self.n_index + 1
	end

	-- TODO: find last relevant event and skip to there?
	-- while self.control_table[self.c_index] and engine.time > self.control_table[self.c_index].time do
	-- 	self.c_index = self.c_index + 1
	-- end
end

function Roll:stop()
	-- any hanging notes shoud get a note off
	for _, note in pairs(self.active_notes) do
		note.is_recording = nil
		local t_offset = engine.time - note.time
		table.insert(note.verts, { t_offset, 0, DEFAULT_PRESSURE })
	end

	self.active_notes = {}
	self.recorded_notes = {}
end

function Roll:playback(channel)
	while self.control_table[self.c_index] and engine.time > self.control_table[self.c_index].time do
		local c = self.control_table[self.c_index]
		if c.name == "sustain" then
			channel:send_event({ name = "sustain", sustain = c.value })
		end
		self.c_index = self.c_index + 1
	end

	while self.note_table[self.n_index] and engine.time > self.note_table[self.n_index].time do
		local note = self.note_table[self.n_index]
		local token = tessera.audio.get_token()
		table.insert(self.voices, { n_index = self.n_index, v_index = 1, token = token })

		channel:send_event({
			name = "note_on",
			token = token,
			interval = note.interval,
			vel = note.vel,
			offset = note.verts[1][2],
		})

		self.n_index = self.n_index + 1
	end

	for i = #self.voices, 1, -1 do
		local v = self.voices[i]
		local note = self.note_table[v.n_index]

		while v.v_index + 1 <= #note.verts and engine.time > note.time + note.verts[v.v_index + 1][1] do
			v.v_index = v.v_index + 1
		end

		-- note off
		if v.v_index >= #note.verts then
			channel:send_event({ name = "note_off", token = v.token })
			table.remove(self.voices, i)
		else
			local t0 = note.verts[v.v_index][1]
			local t1 = note.verts[v.v_index + 1][1]
			local alpha = (engine.time - (note.time + t0)) / (t1 - t0)

			local offset = util.lerp(note.verts[v.v_index][2], note.verts[v.v_index + 1][2], alpha)
			local pressure = util.lerp(note.verts[v.v_index][3], note.verts[v.v_index + 1][3], alpha)

			channel:send_event({ name = "pressure", token = v.token, pressure = pressure })
			channel:send_event({ name = "pitch", token = v.token, offset = offset })
		end
	end
end

-- TODO: cleanup!
function Roll:event(event)
	-- record events to timeline
	if engine.playing and project.transport.recording then
		local time = engine.time

		if event.name == "note_on" then
			local note = {
				interval = event.interval,
				time = time,
				vel = event.vel,
				-- TODO: initial pressure?
				verts = { { 0.0, event.offset, DEFAULT_PRESSURE } },
				is_recording = true,
			}

			self.active_notes[event.token] = note
			table.insert(project.channels[self.ch_index].notes, note)
			table.insert(self.recorded_notes, note)
		elseif event.name == "note_off" then
			local note = self.active_notes[event.token]
			if not note then
				-- Note was pressed before recording started
				return
			end
			note.is_recording = nil
			local t_offset = time - note.time

			local v_prev = note.verts[#note.verts]
			local offset = 0
			local pres = DEFAULT_PRESSURE
			if v_prev then
				offset = v_prev[2]
			end

			table.insert(self.active_notes[event.token].verts, { t_offset, offset, pres })

			self.active_notes[event.token] = nil
		elseif event.name == "pitch" then
			local note = self.active_notes[event.token]
			if not note then
				return
			end
			local t_offset = time - note.time

			local v_prev = note.verts[#note.verts]

			if t_offset - v_prev[1] >= 0.008 then
				local n_new = { t_offset, event.offset, v_prev[3] }
				table.insert(self.active_notes[event.token].verts, n_new)
			else
				note.verts[#note.verts][2] = event.offset
			end
		elseif event.name == "pressure" then
			local note = self.active_notes[event.token]
			if not note then
				return
			end

			local t_offset = time - note.time

			local v_prev = note.verts[#note.verts]

			if t_offset - v_prev[1] >= 0.008 then
				local n_new = { t_offset, v_prev[2], event.pressure }
				table.insert(self.active_notes[event.token].verts, n_new)
			else
				note.verts[#note.verts][3] = event.pressure
			end
		elseif event.name == "sustain" then
			if not project.channels[self.ch_index].control.sustain then
				project.channels[self.ch_index].control.sustain = {}
			end
			local c = { value = event.sustain, time = time }
			table.insert(project.channels[self.ch_index].control.sustain, c)
		else
			print("unhandled event: ", util.dump(event))
		end
	end
end

return Roll
