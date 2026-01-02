local log = require("log")
local tuning = require("tuning")

local midi = {}

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

local MidiDevice = {}
MidiDevice.__index = MidiDevice

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

-- enable or disable a midi connection
function midi.connect(enable, config)
	if enable then
		local index = tessera.midi.open_connection(config.name)
		if index then
			assert(not devices[index])
			devices[index] = MidiDevice.new(config)
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
			local success = midi.connect(true, c)
			if not success then
				-- disable it so we don't try opening it again next scan
				c.enable = false
			end
		end

		-- stale ports to delete
		if (not c.enable or not midi.available_ports[c.name]) and midi.open_ports[c.name] then
			midi.ports_changed = true
			midi.connect(false, c)
		end
	end
end

function midi.update_config(config)
	local device
	for _, v in ipairs(devices) do
		if v.name == config.name then
			device = v
		end
	end

	if device then
		device:set_config(config)
	else
		log.warn(("Device %q not found"):format(config.name))
	end
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
	local sink = {}
	for i, ch in ipairs(ui_channels) do
		if project.channels[i].armed then
			table.insert(sink, ch)
		end
	end

	for _, event in ipairs(events) do
		device:event(sink, event)
	end
end

function midi.quit()
	devices = {}
end

function MidiDevice.new(config)
	local self = setmetatable({}, MidiDevice)

	self:set_config(config)

	self.offset = 0
	self.notes = {}
	self.offsets = {}
	for i = 0, 16 do
		self.offsets[i] = 0
	end
	return self
end

function MidiDevice:set_config(config)
	self.mpe = config.mpe
	self.name = config.name
	self.pitchbend_range = 2
	if self.mpe then
		self.pitchbend_range = 48
	end
end

local function send_event(sink, event)
	for _, v in ipairs(sink) do
		v:event(event)
	end
end

function MidiDevice:event(sink, event)
	if event.name == "note_on" then
		local n_index = event_note_index(event)

		local token = tessera.audio.get_token()
		self.notes[n_index] = token

		local interval = tuning.from_midi(event.note)
		local offset = self.offsets[event.channel]

		send_event(sink, { name = "note_on", token = token, interval = interval, vel = event.vel, offset = offset })
	elseif event.name == "note_off" then
		local n_index = event_note_index(event)
		local token = self.notes[n_index]

		if token then
			send_event(sink, { name = "note_off", token = token })
		else
			log.warn("Unhandled note off event.")
			return
		end

		self.notes[n_index] = nil
	elseif event.name == "pitchbend" then
		local offset = self.pitchbend_range * event.pitchbend
		self.offsets[event.channel] = offset
		if self.mpe then
			for k, token in pairs(self.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					send_event(sink, { name = "pitch", token = token, offset = offset })
				end
			end
		else
			for _, token in pairs(self.notes) do
				send_event(sink, { name = "pitch", token = token, offset = offset })
			end
		end
	elseif event.name == "pressure" then
		if self.mpe then
			for k, token in pairs(self.notes) do
				local midi_ch = math.floor(k / 256)
				if midi_ch == event.channel then
					send_event(sink, { name = "pressure", token = token, pressure = event.pressure })
				end
			end
		else
			-- TODO: untested
			for _, token in pairs(self.notes) do
				send_event(sink, { name = "pressure", token = token, pressure = event.pressure })
			end
		end
	elseif event.name == "controller" then
		if event.controller == 64 then
			-- sustain pedal
			if event.value > 0 then
				send_event(sink, { name = "sustain", sustain = true })
			else
				send_event(sink, { name = "sustain", sustain = false })
			end
		else
			-- send pressure on modwheel (1) or foot pedal (2)
			if event.controller == 1 or event.controller == 2 then
				local p = event.value
				for _, token in pairs(self.notes) do
					send_event(sink, { name = "pressure", token = token, pressure = p })
				end
			end
		end
	end
end

return midi
