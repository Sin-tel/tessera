local Parameter = require("parameter")

local deviceList = {}

deviceList.instruments = {}

deviceList.instruments.sine = {
	index = 0,
	parameters = {
		Parameter:new("feedback", { default = 1.0, max = 1.5 }),
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
		Parameter:new("pulse width", { default = 0.5, min = 0.5, max = 0.99, fmt = "%0.2f" }),
		Parameter:new("mix pulse", { default = -math.huge, t = "dB" }),
		Parameter:new("mix saw", { default = 0, t = "dB" }),
		Parameter:new("mix sub", { default = -math.huge, t = "dB" }),
		Parameter:new("mix noise", { default = -math.huge, t = "dB" }),
		Parameter:new("vcf freq", { default = 2000, min = 20, max = 20000, fmt = "Hz", t = "log" }),
		Parameter:new("vcf res", { default = 0.2, min = 0.0, max = 1.25 }),
		Parameter:new("vcf env", { default = 0.5 }),
		Parameter:new("vcf kbd", { default = 0.5 }),
	},
	mono = true,
}

deviceList.effects = {}

deviceList.effects.pan = {
	index = 0,
	parameters = {
		Parameter:new("gain", { default = -12, t = "dB" }),
		Parameter:new("pan", { default = 0, min = -1, max = 1, centered = true, fmt = "%0.2f" }),
	},
}

deviceList.effects.gain = {
	index = 1,
	parameters = {
		Parameter:new("gain", { default = 0, t = "dB" }),
	},
}

return deviceList
