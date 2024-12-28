local VoiceAlloc = require("voice_alloc")
local engine = require("engine")

local Roll = {}
Roll.__index = Roll

function Roll.new(ch_index)
	local self = setmetatable({}, Roll)

	self.ch_index = ch_index
	self.voice_alloc = ui_channels[ch_index].voice_alloc

	self.n_index = 1
	self.note_table = {}
	self.voices = {}

	self.rec_notes = {}

	return self
end

local function sort_time(a, b)
	return a.time < b.time
end

function Roll:start()
	self.note_table = {}
	self.control_table = {}
	self.voices = {}

	for i, v in ipairs(project.channels[self.ch_index].notes) do
		assert(v.verts[1][1] == 0)
		table.insert(self.note_table, v)
	end
	table.sort(self.note_table, sort_time)

	-- dump all control messages in one table
	-- TODO: don't know if it's good to have two seperate tables
	for k, c in pairs(project.channels[self.ch_index].control) do
		for i, v in ipairs(c) do
			table.insert(self.control_table, { name = k, value = v.value, time = v.time })
		end
	end
	table.sort(self.control_table, sort_time)

	self:seek()
end

function Roll:seek()
	-- seek
	self.n_index = 1
	self.c_index = 1

	-- skip notes already played
	while self.note_table[self.n_index] do
		local note = self.note_table[self.n_index]
		local vt = note.verts[#note.verts][1]
		-- TODO: some annoying edge cases here
		if note.time + vt > project.transport.time then
			break
		end
		self.n_index = self.n_index + 1
	end

	-- TODO: find last relevant event and skip to there?
	-- while self.control_table[self.c_index] and project.transport.time > self.control_table[self.c_index].time do
	-- 	self.c_index = self.c_index + 1
	-- end
end

function Roll:stop()
	-- any hanging notes shoud get a note off
	for _, note in pairs(self.rec_notes) do
		note.is_recording = nil
		local t_offset = project.transport.time - note.time
		table.insert(note.verts, { t_offset, 0, 0.3 })
	end

	self.rec_notes = {}
end

function Roll:playback()
	while self.control_table[self.c_index] and project.transport.time > self.control_table[self.c_index].time do
		local c = self.control_table[self.c_index]
		-- print(util.pprint(c))
		if c.name == "sustain" then
			self.voice_alloc:event({ name = "sustain", sustain = c.value })
		end
		self.c_index = self.c_index + 1
	end

	while self.note_table[self.n_index] and project.transport.time > self.note_table[self.n_index].time do
		local note = self.note_table[self.n_index]
		local id = VoiceAlloc.next_id()
		table.insert(self.voices, { n_index = self.n_index, v_index = 1, id = id })

		self.voice_alloc:event({ name = "note_on", id = id, pitch = note.pitch, vel = note.vel })

		self.n_index = self.n_index + 1
	end

	for i = #self.voices, 1, -1 do
		local v = self.voices[i]
		local note = self.note_table[v.n_index]

		while v.v_index + 1 <= #note.verts and project.transport.time > note.time + note.verts[v.v_index + 1][1] do
			v.v_index = v.v_index + 1
		end

		-- note off
		if v.v_index >= #note.verts then
			self.voice_alloc:event({ name = "note_off", id = v.id })
			table.remove(self.voices, i)
		else
			local t0 = note.verts[v.v_index][1]
			local t1 = note.verts[v.v_index + 1][1]
			local alpha = (project.transport.time - (note.time + t0)) / (t1 - t0)

			local offset = util.lerp(note.verts[v.v_index][2], note.verts[v.v_index + 1][2], alpha)
			local pres = util.lerp(note.verts[v.v_index][3], note.verts[v.v_index + 1][3], alpha)

			self.voice_alloc:event({ name = "cv", id = v.id, offset = offset, pres = pres })
		end
	end
end

function Roll:event(event)
	-- passthrough
	self.voice_alloc:event(event)

	-- record events to timeline
	if engine.playing and project.transport.recording then
		local time = project.transport.time

		if event.name == "note_on" then
			local note = {
				pitch = event.pitch,
				time = time,
				vel = event.vel,
				-- TODO: initial pressure?
				verts = { { 0.0, 0.0, 0.01 } },
				is_recording = true,
			}

			self.rec_notes[event.id] = note
			table.insert(project.channels[self.ch_index].notes, note)
		elseif event.name == "note_off" then
			local note = self.rec_notes[event.id]
			note.is_recording = nil
			local t_offset = time - note.time

			local v_prev = note.verts[#note.verts]
			local offset = 0
			local pres = 0
			if v_prev then
				offset = v_prev[2]
			end

			table.insert(self.rec_notes[event.id].verts, { t_offset, offset, pres })

			self.rec_notes[event.id] = nil
		elseif event.name == "pitch" then
			local note = self.rec_notes[event.id]
			local t_offset = time - note.time

			local v_prev = note.verts[#note.verts]

			local n_new = { t_offset, event.offset, v_prev[3] }
			if t_offset ~= v_prev[1] then
				table.insert(self.rec_notes[event.id].verts, n_new)
			else
				note.verts[#note.verts] = n_new
			end
		elseif event.name == "pressure" then
			local note = self.rec_notes[event.id]
			local t_offset = time - note.time

			local v_prev = note.verts[#note.verts]

			local n_new = { t_offset, v_prev[2], event.pressure }
			if t_offset ~= v_prev[1] then
				table.insert(self.rec_notes[event.id].verts, n_new)
			else
				note.verts[#note.verts] = n_new
			end
		elseif event.name == "sustain" then
			if not project.channels[self.ch_index].control.sustain then
				project.channels[self.ch_index].control.sustain = {}
			end
			local c = { value = event.sustain, time = time }
			table.insert(project.channels[self.ch_index].control.sustain, c)
		else
			print("unhandled event: ", util.pprint(event))
		end
	end
end

return Roll
