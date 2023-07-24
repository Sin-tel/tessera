local tuning = require("tuning")
local backend = require("backend")

local MidiHandler = {}

-- TODO: make this configurable (per intrument?)
-- 0.01 = 40dB dynamic range
local VEL_MIN = 0.02
local LOG_RANGE = -math.log(VEL_MIN)

local function velocity_curve(x)
	local v = x ^ 0.8
	local out = VEL_MIN * math.exp(LOG_RANGE * v)
	return out
end

-- TODO: note is the midi note, make it also have a pitch table
local function newVoice()
	local new = {}
	new.channel = 0
	new.note = 0
	new.offset = 0
	new.vel = 0
	new.pres = 0
	new.age = 0
	new.note_on = false
	return new
end

function MidiHandler:new(n_voices, channel)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.channel = channel

	new.n_voices = n_voices
	-- queue of notes that are held but not active
	new.queue = {}
	-- active voices
	new.voices = {}
	for i = 1, new.n_voices do
		new.voices[i] = newVoice()
	end

	return new
end

-- TODO: handle MPE
-- TODO: always steal when retrigger same midi note
function MidiHandler:event(device, event)
	local channel_index = channelHandler:getChannelIndex(self.channel)
	if event.name == "note on" then
		-- voice stealing logic.
		-- if theres are voices free, use the oldest one,
		-- if not, steal a playing one.
		-- traditionally, priority is given to the oldest voice,
		-- instead, we steal the one closest in pitch, which allows for nicer voice leading.
		local playing_dist, playing_i = 10000, nil
		local released_age, released_i = -1, nil
		for i, v in ipairs(self.voices) do
			v.age = v.age + 1
			local dist = math.abs(v.note - event.note)
			if v.note_on then
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

		local voice = self.voices[new_i]
		if voice.note_on then
			table.insert(self.queue, voice.note)
		end
		voice.note = event.note
		voice.vel = event.vel
		voice.pres = 0
		voice.note_on = true
		voice.age = 0
		local p = tuning:fromMidi(voice.note)
		backend:sendNote(channel_index, p + voice.offset, velocity_curve(voice.vel), new_i)
	elseif event.name == "note off" then
		local get_i
		for i, v in ipairs(self.voices) do
			if v.note == event.note and v.note_on then
				get_i = i
				break
			end
		end
		if get_i == nil then
			-- voice was already dead
			for i, v in ipairs(self.queue) do
				if v == event.note then
					table.remove(self.queue, i)
					break
				end
			end
			return
		end
		local voice = self.voices[get_i]

		if #self.queue == 0 then
			voice.note_on = false
			if not self.sustain then
				-- note off pitches are ignored but we send the correct one anyway
				local p = tuning:fromMidi(voice.note)
				backend:sendNote(channel_index, p + voice.offset, 0, get_i)
			end
		else
			-- pop last note in queue
			voice.note = table.remove(self.queue)
			local p = tuning:fromMidi(voice.note)
			backend:sendNote(channel_index, p + voice.offset, velocity_curve(voice.vel), get_i)
		end
	elseif event.name == "pitchbend" then
		for i, v in ipairs(self.voices) do
			v.offset = device.pitchbend_range * event.offset
			if v.note_on then
				backend:sendCv(channel_index, v.note + v.offset, v.pres, i)
			end
		end
	elseif event.name == "pressure" then
		for i, v in ipairs(self.voices) do
			v.pres = event.pres
			if v.note_on then
				backend:sendCv(channel_index, v.note + v.offset, v.pres, i)
			end
		end
	elseif event.name == "cc" then
		if event.cc == 64 then
			-- sustain pedal
			if event.y > 0 then
				self.sustain = true
			else
				self.sustain = false
				self.queue = {}
				for i, v in ipairs(self.voices) do
					if not v.note_on then
						v.note_on = false
						-- note off pitches are ignored but we send the correct one anyway
						local p = tuning:fromMidi(v.note)
						backend:sendNote(channel_index, p + v.offset, 0, i)
					end
				end
			end
		end
	end
end

return MidiHandler
