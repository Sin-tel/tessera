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
-- TODO: share static state (if we happen to open multiple settings windows)
-- TODO: add a reset button in case backend died

function Settings.new()
	local self = setmetatable({}, Settings)

	self.ui = Ui.new(self)
	self.ui.layout.h = 32
	self.ui.layout:padding(6)
	self.indent = 32

	self.state = {
		host_id = 1,
		device_id = 1,
		buffer_size = 128,
		toggle_buffer = false,
		midi_ports = {},
	}

	-- these don't need to be rebuilt
	self.slider =
		widgets.Slider.new({ min = 64, max = 256, step = 64, default = 128, fmt = "%d samples", no_undo = true })
	self.toggle_buffer_size =
		widgets.Toggle.new("Request buffer size", { style = "checkbox", pad = self.indent, no_undo = true })

	self:rebuild()
	self:rebuild_midi()

	return self
end

function Settings:rebuild_midi()
	self.midi_toggles = {}
	self.state.midi_ports = {}

	for _, v in ipairs(setup.midi_devices) do
		local toggle = widgets.Toggle.new(v.name, { style = "checkbox", pad = self.indent, no_undo = true })
		table.insert(self.midi_toggles, toggle)

		if v.enable then
			self.state.midi_ports[v.name] = 1
		end
	end
end

function Settings:rebuild()
	-- this rebuilds all the widgets and does some queries, so only call when something changed

	-- HOST
	self.hosts = tessera.audio.get_hosts()

	-- build list of hosts for display and find current index
	local host_id = 1
	local host_display_names = {}
	for i, v in ipairs(self.hosts) do
		if v == setup.host then
			host_id = i
		end
		host_display_names[i] = display_name(v)
	end
	self.state.host_id = host_id

	-- build the widget
	self.select_host = widgets.Selector.new({ list = host_display_names, no_undo = true })

	-- DEVICE
	self.devices = tessera.audio.get_output_devices(setup.host)

	-- In case devices is empty we add a dummy value
	if #self.devices == 0 then
		self.devices = { "null" }
	end

	-- build list of devices for display and find current index
	local device_id = 1
	for i, v in ipairs(self.devices) do
		if v == setup.configs[setup.host].device then
			device_id = i
		end
	end
	self.state.device_id = device_id

	if setup.configs[setup.host].buffer_size then
		self.state.toggle_buffer = 1
		self.state.buffer_size = setup.configs[setup.host].buffer_size
	else
		self.state.toggle_buffer = 0
	end

	self.reset_button = widgets.Button.new("Audio offline. Click to reset.")

	-- build the widget
	self.select_device = widgets.Dropdown.new({ list = self.devices, no_undo = true })
end

function Settings:update()
	local lw = math.min(800, self.w - 64)
	local x = 64
	local y = 24

	self.ui:start_frame(x, y)

	local c1 = self.indent
	local c2 = 0.5 * (lw - c1)
	local c3 = c2

	self.ui.layout:col(lw)
	if not tessera.audio.ok() then
		if self.reset_button:update(self.ui) then
			engine.rebuild_stream()
		end
	end
	self.ui.layout:new_row()
	self.ui.layout:col(lw)
	self.ui:label("Audio settings")

	self.ui:background(theme.bg_nested)

	self.ui.layout:new_row()
	self.ui.layout:col(c1, 32)
	self.ui.layout:col(c2)
	self.ui:label("Driver type")
	self.ui.layout:col(c3)
	local host_id = self.select_host:update(self.ui, self.state, "host_id")
	if host_id then
		setup.host = self.hosts[host_id]
		self:rebuild()
		engine.rebuild_stream()
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Output device")
	self.ui.layout:col(c3)
	local device_id = self.select_device:update(self.ui, self.state, "device_id")
	if device_id then
		local new_device = self.devices[device_id]
		if setup.configs[setup.host].device ~= new_device then
			setup.configs[setup.host].device = new_device
			self:rebuild()
			engine.rebuild_stream()
		end
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1 + c2)
	local update_buffer_size = self.toggle_buffer_size:update(self.ui, self.state, "toggle_buffer")

	if self.state.toggle_buffer == 1 then
		self.ui.layout:col(c3)
		local _, commit = self.slider:update(self.ui, self.state, "buffer_size")

		update_buffer_size = update_buffer_size or commit
	end

	if update_buffer_size then
		if self.state.toggle_buffer == 1 then
			setup.configs[setup.host].buffer_size = self.state.buffer_size
		else
			setup.configs[setup.host].buffer_size = nil
		end
		self:rebuild()
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
	local find_id = nil
	for i, v in ipairs(self.hosts) do
		if v == setup.host then
			find_id = i
		end
	end
	if find_id and self.state.host_id ~= find_id then
		self:rebuild()
	end

	-- MIDI

	if midi.ports_changed then
		self:rebuild_midi()
		midi.ports_changed = false
	end

	self.ui:background(theme.background)
	self.ui.layout:new_row()
	self.ui.layout:col(c1 + c2)
	self.ui:label("Midi devices")
	self.ui.layout:col(c3)
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
				local update = self.midi_toggles[i]:update(self.ui, self.state.midi_ports, v.name)
				self.ui.layout:col(c3)

				if update then
					local enable = self.state.midi_ports[v.name] == 1
					setup.midi_devices[i].enable = enable
					if midi.available_ports[v.name] then
						midi.update_port(enable, setup.midi_devices[i])
					end
				end

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
