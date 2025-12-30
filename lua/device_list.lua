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
	name = "Test Sine",
	hide = true,
	parameters = {
		{ "Fixed", "toggle" },
		{ "Freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "Gain", "slider", { default = -12, t = "dB" } },
		{ "Noise", "toggle" },
	},
}

device_list.instruments.wavetable = {
	name = "Wavetable",
	parameters = {
		{ "Position", "slider", { default = 0.0 } },
		{ "Wave", "dropdown", { list = { "Fold", "Glass", "Noise" }, default = 2, arrows = true } },
		{ "Unison", "slider", { default = 0.0 } },
		{ "Animate", "slider", { default = 0.0, min = -1.0, max = 1.0, centered = true } },
		{ "Rate", "slider", { default = 400.0, min = 50.0, max = 3000.0, t = "log", fmt = "ms" } },

		{ "Envelope", "label" },
		-- { "separator" },
		{ "Attack", "slider", { default = 10.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Decay", "slider", { default = 500.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Sustain", "slider", { default = -6, t = "dB" } },
		{ "Release", "slider", { default = 50.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },

		{ "LFO", "label" },
		-- { "separator" },
		{ "Rate", "slider", { default = 0.7, min = 0.01, max = 12.0, t = "log" } },
		{ "Shape", "slider", { default = 0.0 } },
		{ "Random", "slider", { default = 0.0 } },
		{ "Depth", "slider", { default = 0.0 } },
	},
}

device_list.instruments.analog = {
	name = "Analog",
	parameters = {
		{ "Pulse Width", "slider", { default = 0.5, min = 0.5, max = 0.99, fmt = "%0.2f" } },
		{ "Pulse", "slider", { default = -INF, t = "dB" } },
		{ "Saw", "slider", { default = 0, t = "dB" } },
		{ "Sub", "slider", { default = -INF, t = "dB" } },
		{ "Noise", "slider", { default = -30, t = "dB" } },

		-- { "Filter", "label" },
		{ "separator" },
		{ "Filter", "selector", { list = { "Lowpass", "Bandpass", "Highpass" } } },
		{ "Cutoff", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "Resonance", "slider", { default = 0.3, min = 0.0, max = 1.25 } },
		{ "Envelope Mod", "slider", { default = 0.5 } },
		{ "Keytrack", "slider", { default = 0.5 } },

		{ "separator" },
		-- { "Envelope", "label" },
		{ "Gate", "toggle" },
		{ "Attack", "slider", { default = 10.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Decay", "slider", { default = 500.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Sustain", "slider", { default = -6, t = "dB" } },
		{ "Release", "slider", { default = 50.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },

		{ "Legato", "toggle", { default = true } },
	},
}

device_list.instruments.fm = {
	name = "FM",
	parameters = {
		{ "Feedback", "slider", { default = 0.0, min = -1.0, max = 1.0, centered = true } },
		{ "Depth", "slider", { default = 0.2, min = 0, max = 1.0 } },
		{ "Ratio", "slider", { default = 1.0, min = 0.0, max = 8.0, step = 1.0 } },
		{ "Fine", "slider", { default = 0.0 } },
		{ "Offset", "slider", { default = 0.0, min = 0.0, max = 8.0, fmt = "Hz" } },

		{ "separator" },
		{ "Attack", "slider", { default = 10.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Decay", "slider", { default = 500.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Sustain", "slider", { default = -6, t = "dB" } },
		{ "Release", "slider", { default = 50.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },

		{ "separator" },
		{ "Pitch Mod", "slider", { default = 0.0, min = -1.0, max = 1.0, centered = true } },
		{ "Pitch Env", "slider", { default = 20.0, min = 2.0, max = 300.0, t = "log", fmt = "ms" } },

		{ "Keytrack", "slider", { default = 0.4, min = 0.0, max = 1.0 } },
	},
}

device_list.instruments.polysine = {
	name = "Simple Poly",
	parameters = {
		{ "Feedback", "slider", { default = 0.5, max = 2.0 } },
		{ "Attack", "slider", { default = 40.0, min = 1.0, max = 20000.0, t = "log", fmt = "ms" } },
		{ "Release", "slider", { default = 250.0, min = 10.0, max = 20000.0, t = "log", fmt = "ms" } },
	},
}

device_list.instruments.pluck = {
	name = "Pluck",
	parameters = {
		{ "Decay", "slider", { default = 0.66, min = 0.0, max = 1.0 } },
		{ "Release", "slider", { default = 0.5, min = -1.0, max = 1.0, centered = true } },
		{ "Damp", "slider", { default = 0.2, min = 0.0, max = 1.0 } },
		{ "Position", "slider", { default = 0.26, min = 0.1, max = 0.5 } },
		{ "Noise", "slider", { default = 0.4, min = 0.0, max = 1.0 } },
		{ "Dispersion", "slider", { default = 0.25, min = 0.0, max = 1.0 } },
		{ "Bloom", "slider", { default = 0.1, min = 0.0, max = 1.0 } },
	},
}

device_list.instruments.epiano = {
	name = "Epiano",
	parameters = {
		{ "Gain", "slider", { default = -6, min = -24, max = 6, fmt = "%0.1f dB" } },
		{ "Wobble", "slider", { default = 0.3 } },
		{ "Bell", "slider", { default = 0.15 } },
	},
}

device_list.effects = {}

device_list.effects.pan = {
	name = "Pan",
	parameters = {
		{ "Gain", "slider", { default = 0, t = "dB" } },
		{ "Pan", "slider", { default = 0, min = -1, max = 1, centered = true, fmt = "%0.2f" } },
	},
}

device_list.effects.gain = {
	name = "Gain",
	parameters = {
		{ "Gain", "slider", { default = 0, max = 12, t = "dB" } },
	},
}

device_list.effects.wide = {
	name = "Wide",
	parameters = {
		{ "Amount", "slider", { default = 0.2, max = 1 } },
	},
}

device_list.effects.drive = {
	name = "Drive",
	parameters = {
		{ "Dry wet", "slider", { default = 1.0 } },
		{ "Mode", "selector", { list = { "soft", "hard" } } },
		{ "Gain", "slider", { default = 6, min = -6, max = 36 } },
		{ "Post Gain", "slider", { default = 0, min = 0, max = 12 } },
		{ "Bias", "slider", { default = 0.2, max = 1.0 } },
		{ "Tilt", "slider", { default = 0, min = -18, max = 18 } },
		{ "Oversampling", "toggle" },
	},
}

device_list.effects.chorus = {
	name = "Chorus",
	parameters = {
		{ "Dry/Wet", "slider", { default = 1.0 } },
		{ "Rate", "slider", { default = 0.35, min = 0.05, max = 8.0, t = "log" } },
		{ "Depth", "slider", { default = 0.50 } },
		{ "Vibrato", "toggle" },
	},
}

device_list.effects.tremolo = {
	name = "Tremolo",
	parameters = {
		{ "Amount", "slider", { default = 0.4 } },
		{ "Rate", "slider", { default = 1.5, min = 0.50, max = 15.0, t = "log" } },
		{ "Stereo", "slider", { default = 0.5 } },
	},
}

device_list.effects.delay = {
	name = "Delay",
	parameters = {
		{ "Dry/Wet", "slider", { default = 0.33 } },
		{ "Time", "slider", { default = 0.4, min = 0.1, max = 1.0, t = "log" } },
		{ "Offset", "slider", { default = 0.15, min = -1.0, max = 1.0 } },
		{ "Feedback", "slider", { default = 0.66 } },

		{ "separator" },
		{ "LFO Speed", "slider", { default = 1.0, min = 0.2, max = 8.0, fmt = "Hz" } },
		{ "Depth", "slider", { default = 0.15 } },
	},
}

device_list.effects.reverb = {
	name = "Reverb",
	parameters = {
		{ "Dry/Wet", "slider", { default = 0.33 } },
		{ "Size", "slider", { default = 0.8, min = 0.3, max = 1.0 } },
		{ "Decay", "slider", { default = 1.3, min = 0.5, max = 20.0, t = "log", fmt = "s" } },
		{ "Modulation", "slider", { default = 0.5 } },
		{ "Pre-delay", "slider", { default = 0.02, min = 0.0, max = 0.20, fmt = "s" } },
	},
}

device_list.effects.testfilter = {
	name = "Test Filter",
	hide = true,
	parameters = {
		{ "freq", "slider", { default = C5_HZ, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "Q", "slider", { default = DEFAULT_Q, min = 0.5, max = 10, t = "log" } },
		{ "gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "one pole", "toggle" },
	},
}

device_list.effects.equalizer = {
	name = "Equalizer",
	parameters = {
		{ "Low Gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "Band 1 Gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "Band 2 Gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },
		{ "High Gain", "slider", { default = 0, min = -24, max = 24, centered = true, fmt = "%0.1f dB" } },

		{ "separator" },
		{ "Low Freq", "slider", { default = 180, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "Band 1 Freq", "slider", { default = 400, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "Band 2 Freq", "slider", { default = 2000, min = 20, max = 20000, fmt = "Hz", t = "log" } },
		{ "High Freq", "slider", { default = 6500, min = 20, max = 20000, fmt = "Hz", t = "log" } },

		{ "separator" },
		{ "Band 1 Q", "slider", { default = DEFAULT_Q, min = 0.5, max = 5, t = "log" } },
		{ "Band 2 Q", "slider", { default = DEFAULT_Q, min = 0.5, max = 5, t = "log" } },
	},
}

device_list.effects.tilt = {
	name = "Tilt",
	parameters = {
		{ "Slope", "slider", { default = 0, min = -12, max = 12, centered = true, fmt = "%0.1f dB/oct" } },
	},
}

device_list.effects.convolve = {
	name = "Convolution",
	parameters = {
		{ "Dry/Wet", "slider", { default = 1.0 } },
		{
			"Impulse",
			"dropdown",
			{
				list = {
					"Body small",
					"Body medium",
					"Soundboard",
					"Bright Tiles",
					"Small Room",
					"Yard",
					"Large Yard",
					"Living Room",
					"Parking Garage",
					"Nonlinear Space",
				},
				default = 5,
				arrows = true,
			},
		},
		{ "Stereo Width", "slider", { default = 1.0 } },
		{ "Pre-delay", "slider", { default = 0.0, min = 0.0, max = 200.0, fmt = "ms" } },
	},
}

device_list.effects.compressor = {
	name = "Compressor",
	parameters = {
		{ "Dry/Wet", "slider", { default = 1.0 } },
		{ "Treshold", "slider", { default = -24, min = -48, max = 0, fmt = "%0.1f dB" } },
		{ "Ratio", "slider", { default = 4, min = 1, max = 20, t = "log" } },
		{ "Attack", "slider", { default = 0.012, min = 0.001, max = 0.100, t = "log", fmt = "s" } },
		{ "Release", "slider", { default = 0.150, min = 0.010, max = 0.4, fmt = "s" } },
		{ "Make-up Gain", "slider", { default = 0, min = 0, max = 24, fmt = "%0.1f dB" } },
	},
}

return device_list
