require("table.clear")
local Ui = require("ui/ui")
local View = require("view")
local engine = require("engine")
local midi = require("midi")
local widgets = require("ui/widgets")

local Settings = View.derive("Settings")
Settings.__index = Settings

-- fix some capitalization, host_str has to match with backend
local function display_name(host_str)
	if host_str == "alsa" then
		return "ALSA"
	elseif host_str == "asio" then
		return "ASIO"
	elseif host_str == "coreaudio" then
		return "CoreAudio"
	elseif host_str == "jack" then
		return "JACK"
	elseif host_str == "wasapi" then
		return "WASAPI"
	else
		return host_str
	end
end

-- TODO: defer queries to first time we need them

-- cache queries since they can take a long time on some hosts
Settings.hosts = {}
Settings.devices = {}

-- shared ui state
Settings.state = {
	host_id = 1,
	device_id = 1,
	buffer_size = 128,
	toggle_buffer = false,
	midi_ports = {},
	mpe = {},
	devices = {},
}

function Settings.new()
	local self = setmetatable({}, Settings)

	self.ui = Ui.new(self)
	self.ui.layout.h = Ui.scale(32)
	self.ui.layout:padding(6)
	self.indent = Ui.scale(32)

	-- HOST
	if #Settings.hosts == 0 then
		Settings.hosts = tessera.audio.get_hosts()
	end

	-- these don't need to be rebuilt
	self.reset_button = widgets.Button.new("Audio offline. Click to reset.", { text_color = theme.highlight })
	self.control_panel_button = widgets.Button.new("Open control panel")

	self.state.host_id = util.find(Settings.hosts, setup.host) or 1
	self.select_host =
		widgets.Selector.new(self.state, "host_id", { list = util.map(Settings.hosts, display_name), no_undo = true })

	self.select_device = widgets.Dropdown.new(self.state, "device_id", { list = self.state.devices, no_undo = true })

	self.slider = widgets.Slider.new(
		self.state,
		"buffer_size",
		{ min = 64, max = 256, step = 64, default = 128, fmt = "%d samples", no_undo = true }
	)

	self.toggle_buffer_size = widgets.Toggle.new(
		self.state,
		"toggle_buffer",
		{ label = "Request buffer size", style = "checkbox", pad = self.indent, no_undo = true }
	)

	self:rebuild()
	self:rebuild_midi()

	return self
end

function Settings:rebuild_midi()
	self.midi_toggles = {}
	table.clear(self.state.midi_ports)
	self.mpe_toggles = {}
	table.clear(self.state.mpe)

	for _, v in ipairs(setup.midi_devices) do
		-- device on/off toggles
		local toggle = widgets.Toggle.new(
			self.state.midi_ports,
			v.name,
			{ label = v.name, style = "checkbox", pad = self.indent, no_undo = true }
		)
		table.insert(self.midi_toggles, toggle)

		if v.enable then
			self.state.midi_ports[v.name] = true
		end

		-- MPE toggles
		local toggle_mpe = widgets.Toggle.new(self.state.mpe, v.name, { style = "checkbox", no_undo = true })
		table.insert(self.mpe_toggles, toggle_mpe)

		if v.mpe then
			self.state.mpe[v.name] = true
		end
	end
end

function Settings:rebuild()
	-- when host changes, we need to query available devices and sync widget state

	-- DEVICE
	if not Settings.devices[setup.host] then
		Settings.devices[setup.host] = tessera.audio.get_output_devices(setup.host)
	end

	-- update device list in-place
	table.clear(self.state.devices)
	for i, v in ipairs(Settings.devices[setup.host]) do
		self.state.devices[i] = v
	end

	-- In case devices is empty we add a dummy value
	if #self.state.devices == 0 then
		self.state.devices[1] = "null"
	end

	-- build list of devices for display and find current index
	self.state.device_id = util.find(self.state.devices, setup.configs[setup.host].device) or 1

	if setup.configs[setup.host].buffer_size then
		self.state.toggle_buffer = true
		self.state.buffer_size = setup.configs[setup.host].buffer_size
	else
		self.state.toggle_buffer = false
	end
end

function Settings:update()
	local lw = math.min(800, self.w - 64)
	local x = 64
	local y = 24

	self.ui:start_frame(x, y)

	local c1 = self.indent
	local c2 = 0.3 * (lw - c1)
	local c3 = 0.4 * (lw - c1)
	local c4 = 0.3 * (lw - c1)

	local audio_ok = tessera.audio.ok()

	if audio_ok then
		self.ui.layout:col(lw)
	else
		self.ui.layout:col(c1)
		self.ui.layout:col(c2)
		if self.reset_button:update(self.ui) then
			engine.rebuild_stream()
		end
	end
	self.ui.layout:new_row()
	self.ui.layout:col(lw)
	self.ui:label("Audio settings")

	self.ui:background(theme.bg_nested)

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Driver type")
	self.ui.layout:col(c3)
	local host_id = self.select_host:update(self.ui)
	if host_id then
		setup.host = Settings.hosts[host_id]
		-- need to come first or some hosts won't report devices correctly
		self:rebuild()
		engine.rebuild_stream()
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Output device")
	self.ui.layout:col(c3)

	local device_id = self.select_device:update(self.ui)
	if device_id then
		local new_device = self.state.devices[device_id]
		if setup.configs[setup.host].device ~= new_device then
			setup.configs[setup.host].device = new_device
			engine.rebuild_stream()
		end
	end

	if setup.host == "asio" and audio_ok then
		self.ui.layout:col(c4)
		if self.control_panel_button:update(self.ui) then
			tessera.audio.open_control_panel()
		end
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1 + c2)
	local update_buffer_size = self.toggle_buffer_size:update(self.ui)

	if self.state.toggle_buffer then
		self.ui.layout:col(c3)
		local _, commit = self.slider:update(self.ui)

		update_buffer_size = update_buffer_size or commit
	end

	if update_buffer_size then
		if self.state.toggle_buffer then
			setup.configs[setup.host].buffer_size = self.state.buffer_size
		else
			setup.configs[setup.host].buffer_size = nil
		end
		engine.rebuild_stream()
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Buffer size")
	self.ui.layout:col(c3)
	self.ui:label(tostring(engine.buffer_size or "?"))

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Sample rate")
	self.ui.layout:col(c3)
	self.ui:label(tostring(engine.sample_rate or "?"))

	-- check if the device changed due to fallback
	local find_id = util.find(Settings.hosts, setup.host)
	if find_id and self.state.host_id ~= find_id then
		self:rebuild()
	end

	-- MIDI

	c2 = 0.3 * (lw - c1)
	c3 = 0.2 * (lw - c1)
	c4 = 0.4 * (lw - c1)

	if midi.ports_changed then
		self:rebuild_midi()
		midi.ports_changed = false
	end

	self.ui:background(theme.background)
	self.ui.layout:new_row()
	self.ui.layout:col(c1 + c2)
	self.ui:label("Midi devices")
	self.ui.layout:col(c3)
	self.ui:label("MPE")
	self.ui.layout:col(c4)
	self.ui:label("Status")
	self.ui:background(theme.bg_nested)
	if midi.ok then
		if #self.midi_toggles == 0 then
			self.ui.layout:new_row()
			self.ui.layout:col(c1)
			self.ui.layout:col(c2 + c3)
			self.ui:label("No MIDI devices")
		else
			for i, v in ipairs(setup.midi_devices) do
				self.ui.layout:new_row()
				self.ui.layout:col(c1 + c2)
				if self.midi_toggles[i]:update(self.ui) then
					local enable = self.state.midi_ports[v.name]
					setup.midi_devices[i].enable = enable
					if midi.available_ports[v.name] then
						midi.connect(enable, setup.midi_devices[i])
					end
				end
				self.ui.layout:col(c3)
				if self.mpe_toggles[i]:update(self.ui) then
					setup.midi_devices[i].mpe = self.state.mpe[v.name]
					midi.update_config(setup.midi_devices[i])
				end

				self.ui.layout:col(c4)

				if midi.open_ports[v.name] then
					self.ui:label("Active")
				elseif midi.available_ports[v.name] then
					self.ui:label("Disabled", nil, theme.text_dim)
				else
					self.ui:label("Not found", nil, theme.text_dim)
				end
			end
		end
	else
		self.ui.layout:new_row()
		self.ui.layout:col(c1)
		self.ui.layout:col(c2 + c3)
		self.ui:label("MIDI not available")
	end

	self.ui:end_frame()
end

function Settings:draw()
	self.ui:draw()
end

return Settings
