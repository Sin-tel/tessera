local backend = require("backend")
local log = require("log")
local tuning = require("tuning")
local VoiceAlloc = require("voice_alloc")

local midi = {}
local devices = {}

-- unique index for a midi note
local function eventNoteIndex(event)
	return event.channel * 256 + event.note
end

function midi.load(input_ports)
	devices = {}
	backend:midiListPorts()

	for _, v in ipairs(input_ports) do
		-- TODO: 'default' should just open first port

		local name, index = backend:midiOpenConnection(v.name)
		if name then
			assert(not devices[index])
			devices[index] = midi.newDevice(v, name, index)
		end
	end
end

function midi.newDevice(settings, name, index)
	local new = {}
	new.index = index
	new.mpe = settings.mpe
	new.name = name
	new.pitchbend_range = 2
	if new.mpe then
		new.pitchbend_range = 48
	end

	new.notes = {}
	return new
end

function midi.update()
	for _, device in pairs(devices) do
		midi.updateDevice(device)
	end
end

function midi.flush()
	for _, device in pairs(devices) do
		backend:midiPoll(device.index)
	end
end

function midi.updateDevice(device)
	local events = backend:midiPoll(device.index)
	if not events then
		return
	end

	local voice_alloc
	for i, ch in ipairs(ui_channels) do
		if project.channels[i].armed then
			voice_alloc = ch.voice_alloc
			break
		end
	end

	if voice_alloc then
		for _, event in ipairs(events) do
			midi.handle_event(device, voice_alloc, event)
		end
	end
end

function midi.handle_event(device, voice_alloc, event)
	if event.name == "note_on" then
		local n_index = eventNoteIndex(event)
		local id = VoiceAlloc.next_id()
		device.notes[n_index] = id

		local pitch = tuning.getPitch(tuning.fromMidi(event.note))
		local vel = util.velocity_curve(event.vel)

		voice_alloc:noteOn(id, pitch, vel)
	elseif event.name == "note_off" then
		local n_index = eventNoteIndex(event)
		local id = device.notes[n_index]
		if not id then
			log.warn("Unhandled note off event.")
			return
		end

		voice_alloc:noteOff(id)
		device.notes[n_index] = nil
	elseif event.name == "pitchbend" then
		local offset = device.pitchbend_range * event.pitchbend
		for _, id in pairs(device.notes) do
			voice_alloc:pitch(id, offset)
		end
	elseif event.name == "pressure" then
		-- TODO
	elseif event.name == "controller" then
		if event.controller == 64 then
			-- sustain pedal
			if event.value > 0 then
				voice_alloc:setSustain(true)
			else
				voice_alloc:setSustain(false)
			end
		end
	end
end

function midi.quit()
	devices = {}
end

return midi
