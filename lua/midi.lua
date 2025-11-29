local VoiceAlloc = require("voice_alloc")
local log = require("log")
local tuning = require("tuning")

local midi = {}

-- this list should be in sync with tessera.audio midi_connections
local devices = {}

local scan_device_timer = 0

-- unique index for a midi note
local function eventNoteIndex(event)
	return event.channel * 256 + event.note
end

function midi.load()
	devices = {}

	local ports = tessera.midi.ports()
	if #ports == 0 then
		log.info("No midi input ports available.")
	else
		log.info("Available midi input ports:")
		for i, name in ipairs(ports) do
			log.info(" - " .. tostring(i) .. ': "' .. name .. '"')
		end
	end
end

function midi.scanPorts(input_ports)
	-- TODO: 'default' should just open first port

	local available_ports = tessera.midi.ports()
	local devices_open = {}

	for _, v in ipairs(devices) do
		local ok = false
		for _, name in ipairs(available_ports) do
			if name == v.name then
				ok = true
			end
		end
		if ok then
			devices_open[v.config_name] = true
		else
			midi.closeDevice(v.index)
		end
	end

	for _, v in ipairs(input_ports) do
		local config_name = v.name

		if not devices_open[config_name] then
			local name, index = tessera.midi.openConnection(config_name)
			if name then
				assert(not devices[index])
				devices[index] = midi.newDevice(v, name, index, config_name)
			end
		end
	end
end

function midi.closeDevice(index)
	tessera.midi.closeConnection(index)
	table.remove(devices, index)
end

function midi.newDevice(settings, name, index, config_name)
	local new = {}
	new.index = index
	new.mpe = settings.mpe
	new.name = name
	new.config_name = config_name
	new.pitchbend_range = 2
	if new.mpe then
		new.pitchbend_range = 48
	end

	new.notes = {}
	return new
end

function midi.update(dt)
	scan_device_timer = scan_device_timer - dt
	if scan_device_timer < 0 then
		-- scan every .5 seconds
		scan_device_timer = 0.5
		midi.scanPorts(setup.midi.inputs)
	end

	for _, device in ipairs(devices) do
		midi.updateDevice(device)
	end
end

function midi.flush()
	for _, device in ipairs(devices) do
		tessera.midi.poll(device.index)
	end
end

function midi.updateDevice(device)
	local events = tessera.midi.poll(device.index)
	if not events then
		return
	end

	-- TODO: be smarter about the routing here
	local sink
	for i, ch in ipairs(ui_channels) do
		if project.channels[i].armed then
			sink = ch.roll
			break
		end
	end

	if sink then
		for _, event in ipairs(events) do
			midi.event(device, sink, event)
		end
	end
end

function midi.event(device, sink, event)
	if event.name == "note_on" then
		local n_index = eventNoteIndex(event)
		local id = VoiceAlloc.next_id()
		device.notes[n_index] = id

		local pitch = tuning.fromMidi(event.note)

		sink:event({ name = "note_on", id = id, pitch = pitch, vel = event.vel })
	elseif event.name == "note_off" then
		local n_index = eventNoteIndex(event)
		local id = device.notes[n_index]
		if not id then
			log.warn("Unhandled note off event.")
			return
		end

		sink:event({ name = "note_off", id = id })
		device.notes[n_index] = nil
	elseif event.name == "pitchbend" then
		-- TODO: fix mpe pitchbend before note on
		local offset = device.pitchbend_range * event.pitchbend
		if device.mpe then
			for k, id in pairs(device.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					sink:event({ name = "pitch", id = id, offset = offset })
				end
			end
		else
			for _, id in pairs(device.notes) do
				sink:event({ name = "pitch", id = id, offset = offset })
			end
		end
	elseif event.name == "pressure" then
		if device.mpe then
			for k, id in pairs(device.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					sink:event({ name = "pressure", id = id, pressure = event.pressure })
				end
			end
		else
			-- TODO: untested
			for _, id in pairs(device.notes) do
				sink:event({ name = "pressure", id = id, pressure = event.pressure })
			end
		end
	elseif event.name == "controller" then
		if event.controller == 64 then
			-- sustain pedal
			if event.value > 0 then
				sink:event({ name = "sustain", sustain = true })
			else
				sink:event({ name = "sustain", sustain = false })
			end
		end
	end
end

function midi.quit()
	devices = {}
end

return midi
