local device_list = {}

-- should this just define a UI instead of a parameter list?
-- parameters need to have a correct number though
-- TODO: allow for custom UI layouts
-- Add toggle groups
-- Add add headings / separators

local C5_HZ = 523.2511
local INF = math.huge
local DEFAULT_Q = 1 / math.sqrt(2)

device_list.instruments = {}

device_list.instruments.sine = {
	n_voices = 1,
	parameters = {
		{ "fixed", "toggle" },
		{ "freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "gain", "slider", { default = -12, t = "dB" } },
		{ "noise", "toggle" },
	},
}

device_list.instruments.wavetable = {
	n_voices = 1,
	parameters = {
		{ "vel mod", "slider", { default = 0.0 } },
		{ "pres mod", "slider", { default = 0.25 } },
	},
}

device_list.instruments.analog = {
	n_voices = 1,
	parameters = {
		{ "pulse width", "slider", { default = 0.5, min = 0.5, max = 0.99, fmt = "%0.2f" } },
		{ "mix pulse", "slider", { default = -INF, t = "dB" } },
		{ "mix saw", "slider", { default = 0, t = "dB" } },
		{ "mix sub", "slider", { default = -INF, t = "dB" } },
		{ "mix noise", "slider", { default = -30, t = "dB" } },
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

device_list.instruments.fm = {
	n_voices = 16,
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

		{ "pitch mod", "slider", { default = 0.0, min = -1.0, max = 1.0, centered = true } },
		{ "pitch env", "slider", { default = 20.0, min = 2.0, max = 300.0, t = "log", fmt = "ms" } },

		{ "keytrack", "slider", { default = 0.4, min = 0.0, max = 1.0 } },
	},
}

device_list.instruments.polysine = {
	n_voices = 16,
	parameters = {
		{ "feedback", "slider", { default = 0.5, max = 2.0 } },
		{ "attack", "slider", { default = 40.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "release", "slider", { default = 250.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
	},
}

device_list.instruments.pluck = {
	n_voices = 8,
	parameters = {
		{ "decay", "slider", { default = 0.66, min = 0.0, max = 1.0 } },
		{ "release", "slider", { default = 0.5, min = -1.0, max = 1.0, centered = true } },
		{ "damp", "slider", { default = 0.2, min = 0.0, max = 1.0 } },
		{ "position", "slider", { default = 0.26, min = 0.1, max = 0.5 } },
		{ "noise", "slider", { default = 0.4, min = 0.0, max = 1.0 } },
		{ "dispersion", "slider", { default = 0.25, min = 0.0, max = 1.0 } },
		{ "bloom", "slider", { default = 0.1, min = 0.0, max = 1.0 } },
	},
}

device_list.instruments.epiano = {
	n_voices = 16,
	parameters = {
		{ "gain", "slider", { default = -6, min = -24, max = 6, fmt = "%0.1f dB" } },
		{ "wobble", "slider", { default = 0.3 } },
		{ "bell", "slider", { default = 0.15 } },
	},
}

device_list.effects = {}

device_list.effects.pan = {
	parameters = {
		{ "gain", "slider", { default = 0, t = "dB" } },
		{ "pan", "slider", { default = 0, min = -1, max = 1, centered = true, fmt = "%0.2f" } },
	},
}

device_list.effects.gain = {
	parameters = {
		{ "gain", "slider", { default = 0, max = 12, t = "dB" } },
	},
}

device_list.effects.wide = {
	parameters = {
		{ "amount", "slider", { default = 0.2, max = 1 } },
	},
}

device_list.effects.drive = {
	parameters = {
		{ "dry wet", "slider", { default = 1.0 } },
		{ "mode", "selector", { "soft", "hard" } },
		{ "gain", "slider", { default = 6, min = -6, max = 36 } },
		{ "post gain", "slider", { default = 0, min = 0, max = 12 } },
		{ "bias", "slider", { default = 0.2, max = 1.0 } },
		{ "tilt", "slider", { default = 0, min = -18, max = 18 } },
		{ "2x oversample", "toggle" },
	},
}

device_list.effects.delay = {
	parameters = {
		{ "dry wet", "slider", { default = 0.33 } },
		{ "time", "slider", { default = 0.4, min = 0.1, max = 1.0, t = "log" } },
		{ "offset", "slider", { default = 0.15, min = -1.0, max = 1.0 } },
		{ "feedback", "slider", { default = 0.66 } },
		{ "LFO spd", "slider", { default = 1.0, min = 0.2, max = 8.0, fmt = "Hz" } },
		{ "LFO mod", "slider", { default = 0.15 } },
	},
}

device_list.effects.reverb = {
	parameters = {
		{ "dry wet", "slider", { default = 0.33 } },
		{ "size", "slider", { default = 0.8, min = 0.3, max = 1.0 } },
		{ "decay", "slider", { default = 1.3, min = 0.5, max = 20.0, t = "log", fmt = "s" } },
		{ "modulation", "slider", { default = 0.5 } },
		{ "predelay", "slider", { default = 0.02, min = 0.0, max = 0.05, fmt = "s" } },
	},
}

device_list.effects.testfilter = {
	parameters = {
		{ "freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "Q", "slider", { default = DEFAULT_Q, min = 0.5, max = 10, t = "log" } },
		{ "gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "one pole", "toggle" },
	},
}

device_list.effects.equalizer = {
	parameters = {
		{ "low gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "band 1 gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "band 2 gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "high gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "low f", "slider", { default = 180, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "band 1 f", "slider", { default = 400, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "band 2 f", "slider", { default = 2000, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "high f", "slider", { default = 6500, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "band 1 Q", "slider", { default = DEFAULT_Q, min = 0.5, max = 5, t = "log" } },
		{ "band 2 Q", "slider", { default = DEFAULT_Q, min = 0.5, max = 5, t = "log" } },
	},
}

device_list.effects.tilt = {
	parameters = {
		{ "slope", "slider", { default = 0, min = -12, max = 12, centered = true, fmt = "%0.1f dB/oct" } },
	},
}

device_list.effects.convolve = {
	parameters = {
		{ "dry wet", "slider", { default = 1.0 } },
	},
}

device_list.effects.compressor = {
	parameters = {
		{ "dry wet", "slider", { default = 1.0 } },
		{ "treshold", "slider", { default = -24, min = -48, max = 0, fmt = "%0.1f dB" } },
		{ "ratio", "slider", { default = 4, min = 1, max = 20, t = "log" } },
		{ "attack", "slider", { default = 0.012, min = 0.001, max = 0.100, t = "log", fmt = "s" } },
		{ "release", "slider", { default = 0.150, min = 0.010, max = 0.4, fmt = "s" } },
		{ "make up", "slider", { default = 0, min = 0, max = 24, fmt = "%0.1f dB" } },
	},
}

return device_list
