local deviceList = {}

-- TODO: allow for custom UI layouts
-- put spacers or subroupings etc
-- should this just define a UI instead of a parameter list?
-- parameters need to have a correct index though

local C5_HZ = 523.2511

deviceList.instruments = {}

deviceList.instruments.sine = {
	index = 0,
	parameters = {
		{ "fixed", "toggle" },
		{ "freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "gain", "slider", { default = 0, t = "dB" } },
	},
	mono = true,
}

deviceList.instruments.wavetable = {
	index = 1,
	parameters = {},
	mono = true,
}

deviceList.instruments.analog = {
	index = 2,
	parameters = {
		{ "pulse width", "slider", { default = 0.5, min = 0.5, max = 0.99, fmt = "%0.2f" } },
		{ "mix pulse", "slider", { default = -math.huge, t = "dB" } },
		{ "mix saw", "slider", { default = 0, t = "dB" } },
		{ "mix sub", "slider", { default = -math.huge, t = "dB" } },
		{ "mix noise", "slider", { default = -math.huge, t = "dB" } },
		{ "vcf mode", "selector", { "lowpass", "bandpass", "highpass" } },
		{ "vcf freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "vcf res", "slider", { default = 0.2, min = 0.0, max = 1.25 } },
		{ "vcf env", "slider", { default = 0.5 } },
		{ "vcf kbd", "slider", { default = 0.5 } },
	},
	mono = true,
}

deviceList.instruments.fm = {
	index = 3,
	parameters = {
		{ "feedback", "slider", { default = 0.0, min = -1.2, max = 1.2, centered = true } },
	},
	mono = true,
}

deviceList.effects = {}

deviceList.effects.pan = {
	index = 0,
	parameters = {
		{ "gain", "slider", { default = -12, t = "dB" } },
		{ "pan", "slider", { default = 0, min = -1, max = 1, centered = true, fmt = "%0.2f" } },
	},
}

deviceList.effects.gain = {
	index = 1,
	parameters = {
		{ "gain", "slider", { default = 0, t = "dB" } },
	},
}

return deviceList
