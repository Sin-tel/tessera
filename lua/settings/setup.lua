local setup = {
	audio = {
		default_host = "ASIO",
		default_device = "asio4all",
		buffer_size = 128,
	},
	midi = {
		inputs = {
			{ name = "sl studio", mpe = false },
			{ name = "loop", mpe = false },
			{ name = "um-1", mpe = false },
			{ name = "seaboard", mpe = true },
			{ name = "linnstrument", mpe = true },
		},
	},
}
return setup
