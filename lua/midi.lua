local log = require("log")
local tuning = require("tuning")

local midi = {}

-- this list should be in sync with tessera.audio midi_connections
local devices = {}

local scan_device_timer = 0

-- unique index for a midi note
local function event_note_index(event)
	return event.channel * 256 + event.note
end

function midi.load()
	midi.ok = tessera.midi.status()

	if midi.ok then
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
	else
		log.info("Midi failed to intialize. Proceeding without.")
	end
end

function midi.scan_ports(input_ports)
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
			midi.close_device(v.index)
		end
	end

	for _, v in ipairs(input_ports) do
		local config_name = v.name

		if not devices_open[config_name] then
			local name, index = tessera.midi.open_connection(config_name)
			if name then
				assert(not devices[index])
				devices[index] = midi.new_device(v, name, config_name)
			end
		end
	end
end

function midi.close_device(index)
	tessera.midi.close_connection(index)
	table.remove(devices, index)
end

function midi.new_device(settings, name, config_name)
	local new = {}
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
	if not midi.ok then
		return
	end

	scan_device_timer = scan_device_timer - dt
	if scan_device_timer < 0 then
		-- scan every .5 seconds
		scan_device_timer = 0.5
		midi.scan_ports(setup.midi_ports)
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

		sink:event({ name = "note_on", token = token, pitch = pitch, vel = event.vel })
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
		local offset = device.pitchbend_range * event.pitchbend
		if device.mpe then
			for k, token in pairs(device.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					sink:event({ name = "pitch", token = token, offset = offset })
				end
			end
		else
			for _, token in pairs(device.notes) do
				sink:event({ name = "pitch", token = token, offset = offset })
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
