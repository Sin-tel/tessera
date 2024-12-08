local VoiceAlloc = require("voice_alloc")
local tuning = require("tuning")

local Roll = {}
Roll.__index = Roll

function Roll.new(ch_index)
	local self = setmetatable({}, Roll)

	self.ch_index = ch_index
	self.voice_alloc = ui_channels[ch_index].voice_alloc

	self.n_index = 1
	self.note_table = {}
	self.voices = {}

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

		-- note on
		local p = tuning.getPitch(note.pitch)
		local vel = util.velocity_curve(note.vel)

		self.voice_alloc:noteOn(id, p, vel)

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
			self.voice_alloc:noteOff(v.id)
			table.remove(self.voices, i)
		else
			local t0 = note.verts[v.v_index][1]
			local t1 = note.verts[v.v_index + 1][1]
			local alpha = (project.transport.time - (note.time + t0)) / (t1 - t0)

			local p_off = util.lerp(note.verts[v.v_index][2], note.verts[v.v_index + 1][2], alpha)
			local press = util.lerp(note.verts[v.v_index][3], note.verts[v.v_index + 1][3], alpha)

			self.voice_alloc:cv(v.id, p_off, press)
		end
	end
end

return Roll
