local tuning = require("tuning")
local backend = require("backend")

local MidiHandler = {}

-- TODO: make this configurable (per intrument?)
-- 0.01 = 40dB dynamic range
-- 0.02 = 34dB dynamic range
-- 0.05 = 26dB dynamic range
-- 0.10 = 20dB dynamic range
local VEL_MIN = 0.05
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

function MidiHandler:new(n_voices, ch_index)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.ch_index = ch_index

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

-- TODO: always steal when retrigger same midi note
-- TODO: fix mpe pitch before playing note
function MidiHandler:event(device, event)
	local mpe = device.mpe

	if event.name == "note_on" then
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
		-- if voice.note_on then
		-- 	table.insert(self.queue, util.deepcopy(voice))
		-- end
		voice.note = event.note
		voice.vel = event.vel
		voice.channel = event.channel
		voice.pres = 0
		voice.note_on = true
		voice.age = 0

		local p = tuning.getPitch(tuning.fromMidi(voice.note))
		print("note_on", self.ch_index, p + voice.offset, velocity_curve(voice.vel), new_i)

		backend:sendNote(self.ch_index, p + voice.offset, velocity_curve(voice.vel), new_i)
	elseif event.name == "note_off" then
		local get_i
		for i, v in ipairs(self.voices) do
			if v.note == event.note and v.note_on then
				get_i = i
				break
			end
		end
		if get_i == nil then
			-- voice was already dead
			-- for i, v in ipairs(self.queue) do
			-- 	if v.note == event.note and v.channel == event.channel then
			-- 		table.remove(self.queue, i)
			-- 		break
			-- 	end
			-- end
			return
		end
		local voice = self.voices[get_i]

		if #self.queue == 0 then
			voice.note_on = false
			if not self.sustain then
				-- note off pitches are ignored but we send the correct one anyway
				local p = tuning.getPitch(tuning.fromMidi(voice.note))
				print("note_off", self.ch_index, p + voice.offset, 0, get_i)
				backend:sendNote(self.ch_index, p + voice.offset, 0, get_i)
			end
		else
			print("ERROR")
			-- pop last note in queue
			-- local old_voice = table.remove(self.queue)
			-- voice.note = old_voice.note
			-- voice.channel = old_voice.channel
			-- voice.vel = old_voice.vel
			-- voice.pres = old_voice.pres
			-- local p = tuning.fromMidi(voice.note)
			-- backend:sendNote(self.ch_index, p + voice.offset, velocity_curve(voice.vel), get_i)
		end
	elseif event.name == "pitchbend" then
		if mpe then
			for i, v in ipairs(self.voices) do
				if v.channel == event.channel then
					v.offset = device.pitchbend_range * event.pitchbend
					if v.note_on then
						backend:sendCv(self.ch_index, v.note + v.offset, v.pres, i)
					end
				end
			end
		else
			for i, v in ipairs(self.voices) do
				v.offset = device.pitchbend_range * event.pitchbend
				if v.note_on then
					backend:sendCv(self.ch_index, v.note + v.offset, v.pres, i)
				end
			end
		end
	elseif event.name == "pressure" then
		if mpe then
			for i, v in ipairs(self.voices) do
				if v.channel == event.channel then
					v.pres = event.pressure
					if v.note_on then
						-- print("pres", self.ch_index, v.note + v.offset, v.pres, i)

						backend:sendCv(self.ch_index, v.note + v.offset, v.pres, i)
					end
				end
			end
		else
			for i, v in ipairs(self.voices) do
				v.pres = event.pressure
				if v.note_on then
					backend:sendCv(self.ch_index, v.note + v.offset, v.pres, i)
				end
			end
		end
	elseif event.name == "controller" then
		if event.controller == 64 then
			-- sustain pedal
			if event.value > 0 then
				self.sustain = true
			else
				self.sustain = false
				self.queue = {}
				for i, v in ipairs(self.voices) do
					if not v.note_on then
						v.note_on = false
						-- note off pitches are ignored but we send the correct one anyway
						local p = tuning.getPitch(tuning.fromMidi(v.note))
						backend:sendNote(self.ch_index, p + v.offset, 0, i)
					end
				end
			end
		end
	end
end

return MidiHandler
