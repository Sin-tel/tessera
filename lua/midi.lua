local log = require("log")
local tuning = require("tuning")

local midi = {}

-- TODO: other config stuff like MPE

-- used by Settings to check if list should be updated
midi.ports_changed = false
-- set of currently open ports
midi.open_ports = {}
-- set of currently available ports
midi.available_ports = {}

-- this list should be in sync with backend midi_connections
local devices = {}

local scan_timer = 0

-- unique index for a midi note
local function event_note_index(event)
	return event.channel * 256 + event.note
end

function midi.load()
	if midi.ok then
		return
	end
	devices = {}
	midi.ok = tessera.midi.init()

	if not midi.ok then
		log.info("Midi failed to intialize.")
	end
end

function midi.update_port(enable, config)
	if enable then
		local index = tessera.midi.open_connection(config.name)
		if index then
			assert(not devices[index])
			devices[index] = midi.new_device(config)
			midi.open_ports[config.name] = true
			return true
		else
			log.warn(("Midi device %q not found"):format(config.name))
			return false
		end
	else
		local index = tessera.midi.close_connection(config.name)
		if index then
			assert(devices[index])
			table.remove(devices, index)
			midi.open_ports[config.name] = nil
			return true
		else
			log.warn(("Midi device %q not found"):format(config.name))
			return false
		end
	end
end

function midi.scan_ports()
	local available_ports = tessera.midi.ports()

	midi.available_ports = {}
	for _, name in ipairs(available_ports) do
		midi.available_ports[name] = true

		-- check if it is preset in setup
		local found = false
		for _, c in ipairs(setup.midi_devices) do
			if name == c.name then
				found = true
				break
			end
		end

		-- if not, add it to setup
		if not found then
			midi.ports_changed = true
			log.info(("Found new midi device: %q"):format(name))
			table.insert(setup.midi_devices, { name = name })
		end
	end

	for _, c in ipairs(setup.midi_devices) do
		-- new ports to add
		if c.enable and midi.available_ports[c.name] and not midi.open_ports[c.name] then
			midi.ports_changed = true
			local success = midi.update_port(true, c)
			if not success then
				-- disable it so we don't try opening it again next scan
				c.enable = false
			end
		end

		-- stale ports to delete
		if (not c.enable or not midi.available_ports[c.name]) and midi.open_ports[c.name] then
			midi.ports_changed = true
			midi.update_port(false, c)
		end
	end
end

function midi.new_device(config)
	local new = {}
	new.mpe = config.mpe
	new.name = config.name
	new.pitchbend_range = 2
	if new.mpe then
		new.pitchbend_range = 48
	end

	new.offset = 0

	new.notes = {}
	return new
end

function midi.update(dt)
	if not midi.ok then
		return
	end

	scan_timer = scan_timer - dt
	if scan_timer < 0 then
		-- scan every .5 seconds
		scan_timer = 0.5
		midi.scan_ports()
	end

	for i, device in ipairs(devices) do
		midi.update_device(i, device)
	end
end

function midi.flush()
	-- clear buffers
	for i in ipairs(devices) do
		tessera.midi.poll(i)
	end
end

function midi.update_device(device_index, device)
	local events = tessera.midi.poll(device_index)
	if not events then
		return
	end

	-- TODO: be smarter about the routing here
	local sink
	for i, ch in ipairs(ui_channels) do
		if project.channels[i].armed then
			sink = ch
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
		local n_index = event_note_index(event)
		local token = tessera.audio.get_token()
		device.notes[n_index] = token

		local pitch = tuning.from_midi(event.note)

		sink:event({ name = "note_on", token = token, pitch = pitch, vel = event.vel, offset = device.offset })
	elseif event.name == "note_off" then
		local n_index = event_note_index(event)
		local token = device.notes[n_index]
		if not token then
			log.warn("Unhandled note off event.")
			return
		end

		sink:event({ name = "note_off", token = token })
		device.notes[n_index] = nil
	elseif event.name == "pitchbend" then
		-- TODO: fix mpe pitchbend before note on
		device.offset = device.pitchbend_range * event.pitchbend
		if device.mpe then
			for k, token in pairs(device.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					sink:event({ name = "pitch", token = token, offset = device.offset })
				end
			end
		else
			for _, token in pairs(device.notes) do
				sink:event({ name = "pitch", token = token, offset = device.offset })
			end
		end
	elseif event.name == "pressure" then
		if device.mpe then
			for k, token in pairs(device.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					sink:event({ name = "pressure", token = token, pressure = event.pressure })
				end
			end
		else
			-- TODO: untested
			for _, token in pairs(device.notes) do
				sink:event({ name = "pressure", token = token, pressure = event.pressure })
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
