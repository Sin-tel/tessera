local VoiceAlloc = require("voice_alloc")
local tuning = require("tuning")
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

function Roll:start()
	self.note_table = {}
	self.voices = {}

	for i, v in ipairs(project.channels[self.ch_index].notes) do
		assert(v.verts[1][1] == 0)
		table.insert(self.note_table, v)
	end
	table.sort(self.note_table, function(a, b)
		return a.time < b.time
	end)

	-- seek
	self.n_index = 1

	while self.note_table[self.n_index] and project.transport.time > self.note_table[self.n_index].time do
		self.n_index = self.n_index + 1
	end
end

function Roll:playback()
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

	if engine.playing and project.transport.recording then
		if event.name == "note_on" then
			local note = {
				pitch = event.pitch,
				time = project.transport.time,
				vel = event.vel,
				verts = { { 0.0, 0.0, 0.3 } },
			}

			self.rec_notes[event.id] = note
			table.insert(project.channels[self.ch_index].notes, note)
		elseif event.name == "note_off" then
			local note = self.rec_notes[event.id]
			local t_offset = project.transport.time - note.time
			table.insert(self.rec_notes[event.id].verts, { t_offset, 0, 0.3 })
		end
	end
end

return Roll
