--[[
When first starting the program `setup.lua` will be generated in this folder, edit it to look something like this.

None of these strings are case-sensitive, and they will also match substrings.
For example, the default audio out on my laptop is called 'Speakers (Realtek(R) Audio)', which I can use by setting:
	default_device = 'speakers'

You can also use the string 'default' to specify that the system default host or device should be used.
]]

local setup = {
	audio = {
		--[[
		Audio backend
		common values are:
			- 'WASAPI' or 'ASIO' on windows
			- 'ALSA' or 'PULSE' on linux
		]]
		default_host = "ASIO",
		-- Name of the output device.
		default_device = "asio4all",
		-- Requested buffer size. The actual buffer size may be different.
		buffer_size = 128,
	},
	midi = {
		inputs = {
			{ name = "keyboard", mpe = false },
			{ name = "linnstrument", mpe = true },
		},
	},
}
return setup
