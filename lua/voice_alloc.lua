local bit = require("bit")
local backend = require("backend")
local log = require("log")

local VoiceAlloc = {}

VoiceAlloc.__index = VoiceAlloc

local id_ = 0

-- TODO: should store tuning coords
-- TODO: figure out a nice way to get events to record
local function newVoice()
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

function VoiceAlloc.new(channel_index, n_voices)
	local self = setmetatable({}, VoiceAlloc)

	self.channel_index = channel_index
	self.n_voices = n_voices
	-- queue of notes that are held but not active
	self.queue = {}

	self.voices = {}
	for i = 1, self.n_voices do
		self.voices[i] = newVoice()
	end

	return self
end

function VoiceAlloc:findVoice(id)
	for i, v in ipairs(self.voices) do
		if v.id == id then
			return i, v
		end
	end
end

function VoiceAlloc:noteOn(id, pitch, vel)
	-- voice stealing logic
	-- if theres are voices free, use the oldest one,
	-- if not, steal a playing one. Priority goes to closest one in pitch
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
		print("add to queue")
		table.insert(self.queue, self.voices[new_i])
		self.voices[new_i] = newVoice()
	end

	local voice = self.voices[new_i]
	voice.id = id
	voice.pitch = pitch
	voice.vel = vel
	voice.offset = 0
	voice.pres = 0.1
	voice.age = 0
	voice.key_down = true

	backend:noteOn(self.channel_index, pitch, vel, new_i)
end

function VoiceAlloc:noteOff(id)
	local i, v = self:findVoice(id)
	if not v then
		-- voice was already dead
		for j, b in ipairs(self.queue) do
			if b.id == id then
				print("removed from queue")
				table.remove(self.queue, j)
				break
			end
		end
		return
	end

	if #self.queue == 0 then
		v.key_down = false
		if not self.sustain then
			backend:noteOff(self.channel_index, i)
		end
	else
		-- pop
		local old_voice = table.remove(self.queue)
		self.voices[i] = old_voice
		backend:noteOn(self.channel_index, old_voice.pitch, old_voice.vel, i)
	end
end

function VoiceAlloc:cv(id, offset, pres)
	local i, v = self:findVoice(id)
	if not v then
		return
	end
	v.offset = offset
	v.pres = pres
	backend:sendCv(self.channel_index, v.pitch + v.offset, pres, i)
end

function VoiceAlloc:pitch(id, offset)
	local i, v = self:findVoice(id)
	if not v then
		return
	end
	v.offset = offset
	backend:sendCv(self.channel_index, v.pitch + v.offset, v.pres, i)
end

function VoiceAlloc:setSustain(s)
	self.sustain = s
	if not s then
		for i, v in ipairs(self.voices) do
			if not v.key_down then
				v.key_down = false
				backend:noteOff(self.channel_index, i)
			end
		end
	end
end

function VoiceAlloc:allNotesOff()
	for i, v in ipairs(self.voices) do
		self.sustain = false
		if v.key_down then
			backend:noteOff(self.channel_index, i)
		end
		self.voices[i] = newVoice()
	end
end

return VoiceAlloc
