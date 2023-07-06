local deviceList = {}

-- should this just define a UI instead of a parameter list?
-- parameters need to have a correct index though
-- TODO: allow for custom UI layouts
-- Add toggle groups
-- Add add headings / separators

local C5_HZ = 523.2511
local INF = math.huge

deviceList.instruments = {}

deviceList.instruments.sine = {
	index = 0,
	parameters = {
		{ "fixed", "toggle" },
		{ "freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "gain", "slider", { default = 0, t = "dB" } },
	},
}

deviceList.instruments.wavetable = {
	index = 1,
	parameters = {
		{ "vel mod", "slider", { default = 0.0 } },
		{ "pres mod", "slider", { default = 0.25 } },
	},
}

deviceList.instruments.analog = {
	index = 2,
	parameters = {
		{ "pulse width", "slider", { default = 0.5, min = 0.5, max = 0.99, fmt = "%0.2f" } },
		{ "mix pulse", "slider", { default = -INF, t = "dB" } },
		{ "mix saw", "slider", { default = 0, t = "dB" } },
		{ "mix sub", "slider", { default = -INF, t = "dB" } },
		{ "mix noise", "slider", { default = -INF, t = "dB" } },
		{ "vcf mode", "selector", { "lowpass", "bandpass", "highpass" } },
		{ "vcf freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "vcf res", "slider", { default = 0.3, min = 0.0, max = 1.25 } },
		{ "vcf env", "slider", { default = 0.5 } },
		{ "vcf kbd", "slider", { default = 0.5 } },

		{ "gate", "toggle" },
		{ "attack", "slider", { default = 10.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "decay", "slider", { default = 500.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "sustain", "slider", { default = -6, t = "dB" } },
		{ "release", "slider", { default = 50.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },

		{ "legato", "toggle", { default = true } },
	},
}

deviceList.instruments.fm = {
	index = 3,
	parameters = {
		{ "feedback", "slider", { default = 0.0, min = -1.0, max = 1.0, centered = true } },
		{ "depth", "slider", { default = 0.2, min = 0, max = 1.0 } },
		{ "ratio", "slider", { default = 1.0, min = 0.0, max = 8.0, step = 1.0 } },
		{ "fine", "slider", { default = 0.0 } },
		{ "offset", "slider", { default = 0.0, min = 0.0, max = 8.0, fmt = "Hz" } },

		{ "attack", "slider", { default = 10.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "decay", "slider", { default = 500.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "sustain", "slider", { default = -6, t = "dB" } },
		{ "release", "slider", { default = 50.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },

		{ "noise", "slider", { default = 0.0, min = 0.0, max = 1.0 } },
		{ "noise env", "slider", { default = 20.0, min = 2.0, max = 5000.0, t = "log", fmt = "ms" } },
	},
}

deviceList.instruments.polysine = {
	index = 4,
	parameters = {
		{ "feedback", "slider", { default = 0.5 } },
		{ "attack", "slider", { default = 10.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "release", "slider", { default = 50.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
	},
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
		{ "gain", "slider", { default = 0, max = 12, t = "dB" } },
	},
}

deviceList.effects.drive = {
	index = 2,
	parameters = {
		{ "mode", "selector", { "soft", "hard" } },
		{ "gain", "slider", { default = 6, max = 24, t = "dB" } },
		{ "bias", "slider", { default = 0, max = 1.0 } },
		{ "2x oversample", "toggle" },
	},
}

return deviceList
