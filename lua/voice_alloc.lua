local bit = require("bit")
local log = require("log")
local tuning = require("tuning")

local VoiceAlloc = {}

VoiceAlloc.__index = VoiceAlloc

local id_ = 0

local function new_voice()
	local new = {}
	new.id = 0
	new.pitch = 0
	new.vel = 0
	new.offset = 0
	new.pres = 0
	new.age = 0
	new.key_down = false
	return new
end

function VoiceAlloc.next_id()
	-- explicit 32 bit wrapping
	id_ = bit.tobit(id_ + 1)
	return id_
end

function VoiceAlloc.new(ch_index, n_voices)
	local self = setmetatable({}, VoiceAlloc)

	self.ch_index = ch_index
	self.n_voices = n_voices
	-- queue of notes that are held but not active
	self.queue = {}

	self.voices = {}
	for i = 1, self.n_voices do
		self.voices[i] = new_voice()
	end

	return self
end

function VoiceAlloc:find_voice(id)
	for i, v in ipairs(self.voices) do
		if v.id == id then
			return i, v
		end
	end
end

function VoiceAlloc:event(event)
	if event.name == "note_on" then
		self:note_on(event.id, event.pitch, event.vel)
	elseif event.name == "note_off" then
		self:note_off(event.id)
	elseif event.name == "cv" then
		self:cv(event.id, event.offset, event.pres)
	elseif event.name == "pitch" then
		self:pitch(event.id, event.offset)
	elseif event.name == "pressure" then
		self:pressure(event.id, event.pressure)
	elseif event.name == "sustain" then
		self:set_sustain(event.sustain)
	else
		log.warn("unhandled event: ", util.pprint(event))
	end
end

function VoiceAlloc:note_on(id, pitch_coord, vel)
	-- voice stealing logic
	-- if theres are voices free, use the oldest one,
	-- if not, steal a playing one. Priority goes to closest one in pitch

	local pitch = tuning.get_pitch(pitch_coord)

	local playing_dist, playing_i = math.huge, nil
	local released_age, released_i = -1, nil
	for i, v in ipairs(self.voices) do
		v.age = v.age + 1
		local dist = math.abs(v.pitch - pitch)
		if v.key_down then
			-- track note is on
			if dist < playing_dist then
				playing_i = i
				playing_dist = dist
			end
		else
			-- track is free
			if v.age > released_age then
				released_i = i
				released_age = v.age
			end
		end
	end
	local new_i
	if released_i ~= nil then
		new_i = released_i
	else
		new_i = playing_i
	end

	assert(new_i ~= nil)

	if self.voices[new_i].key_down then
		table.insert(self.queue, self.voices[new_i])
		self.voices[new_i] = new_voice()
	end

	local voice = self.voices[new_i]
	voice.id = id
	voice.pitch = pitch
	voice.vel = vel
	voice.offset = 0
	voice.pres = 0.1
	voice.age = 0
	voice.key_down = true

	local v_curve = util.velocity_curve(vel)
	tessera.audio.note_on(self.ch_index, pitch, v_curve, new_i)
end

function VoiceAlloc:note_off(id)
	local i, v = self:find_voice(id)
	if not v then
		-- voice was already dead
		for j, b in ipairs(self.queue) do
			if b.id == id then
				table.remove(self.queue, j)
				break
			end
		end
		return
	end

	if #self.queue == 0 then
		v.key_down = false
		if not self.sustain then
			tessera.audio.note_off(self.ch_index, i)
		end
	else
		-- pop
		local old_voice = table.remove(self.queue)
		self.voices[i] = old_voice
		tessera.audio.note_on(self.ch_index, old_voice.pitch, old_voice.vel, i)
	end
end

function VoiceAlloc:cv(id, offset, pres)
	local i, v = self:find_voice(id)
	if not v then
		return
	end
	v.offset = offset
	v.pres = pres
	tessera.audio.pitch(self.ch_index, v.pitch + v.offset, i)
	tessera.audio.pressure(self.ch_index, pres, i)
end

function VoiceAlloc:pitch(id, offset)
	local i, v = self:find_voice(id)
	if not v then
		return
	end
	v.offset = offset
	tessera.audio.pitch(self.ch_index, v.pitch + v.offset, i)
end

function VoiceAlloc:pressure(id, pres)
	local i, v = self:find_voice(id)
	if not v then
		return
	end
	v.pres = pres
	tessera.audio.pressure(self.ch_index, v.pres, i)
end

function VoiceAlloc:set_sustain(s)
	self.sustain = s
	if not s then
		for i, v in ipairs(self.voices) do
			if not v.key_down then
				v.key_down = false
				tessera.audio.note_off(self.ch_index, i)
			end
		end
	end
end

function VoiceAlloc:all_notes_off()
	self:set_sustain(false)
	for i, v in ipairs(self.voices) do
		if v.key_down then
			tessera.audio.note_off(self.ch_index, i)
		end
		self.voices[i] = new_voice()
	end
end

return VoiceAlloc
