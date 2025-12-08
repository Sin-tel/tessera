local Ui = require("ui/ui")
local View = require("view")
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

function Settings.new()
	local self = setmetatable({}, Settings)

	self.ui = Ui.new(self)
	self.ui.layout.h = 32
	self.ui.layout:padding(6)

	-- audio device setup

	-- In cpal these are called hosts, but for the menu we will call them "drivers"
	local default_driver, drivers = tessera.audio.get_hosts()

	-- use first one by if there is no proper default, though cpal should always return a valid one
	local host_id = 1
	local driver_display_names = {}
	for i, v in ipairs(drivers) do
		if default_driver == v then
			host_id = i
		end
		driver_display_names[i] = display_name(v)
	end

	local default_device, devices = tessera.audio.get_output_devices(drivers[host_id])

	local device_id = 1
	for i, v in ipairs(devices) do
		if default_device == v then
			device_id = i
		end
	end

	-- midi device setup

	self.midi_ports = tessera.midi.ports()

	self.state = {
		driver = host_id,
		output_device = device_id,
		buffer_size = 128,
		toggle_buffer = false,
		midi_ports = {},
	}

	self.indent = 32

	-- self.select_driver = widgets.Dropdown.new({ list = driver_display_names,  no_undo = true })
	self.select_driver = widgets.Selector.new({ list = driver_display_names, no_undo = true })
	self.select_device = widgets.Dropdown.new({ list = devices, no_undo = true })
	self.slider =
		widgets.Slider.new({ min = 64, max = 256, step = 64, default = 128, fmt = "%d samples", no_undo = true })
	self.toggle_buffer_size =
		widgets.Toggle.new("Request buffer size", { style = "checkbox", pad = self.indent, no_undo = true })

	self.midi_toggles = {}
	for _, v in ipairs(self.midi_ports) do
		local toggle = widgets.Toggle.new(v, { style = "checkbox", pad = self.indent, no_undo = true })
		table.insert(self.midi_toggles, toggle)
	end

	return self
end

function Settings:update()
	local lw = math.min(800, self.w - 64)
	local x = 64
	local y = 32

	self.ui:start_frame(x, y)

	local c1 = self.indent
	local c2 = 0.5 * (lw - c1)
	local c3 = c2

	self.ui.layout:col(lw)
	self.ui:label("Audio settings")
	self.ui:background(theme.bg_nested)

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Driver type")
	self.ui.layout:col(c3)
	self.select_driver:update(self.ui, self.state, "driver")

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Output device")
	self.ui.layout:col(c3)
	self.select_device:update(self.ui, self.state, "output_device")

	self.ui.layout:new_row()
	self.ui.layout:col(c1 + c2)
	self.toggle_buffer_size:update(self.ui, self.state, "toggle_buffer")

	if self.state.toggle_buffer == 1 then
		self.ui.layout:col(c3)
		self.slider:update(self.ui, self.state, "buffer_size")
	end

	self.ui.layout:new_row()
	self.ui.layout:col(lw)
	self.ui:background(theme.background)
	self.ui:label("Midi devices")

	self.ui:background(theme.bg_nested)
	for _, v in ipairs(self.midi_toggles) do
		self.ui.layout:new_row()
		self.ui.layout:col(c1 + c2)
		v:update(self.ui, self.state.midi_ports, v.text)
		self.ui.layout:col(c3)
		if self.state.midi_ports[v.text] == 1 then
			self.ui:label("OK")
		else
			self.ui:label("disabled", nil, theme.text_dim)
		end
	end

	self.ui:end_frame()
end

function Settings:draw()
	self.ui:draw()
end

return Settings
