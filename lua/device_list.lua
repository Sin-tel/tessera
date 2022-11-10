local Parameter = require("parameter")

local deviceList = {}

deviceList.channel = {
	Parameter:new("gain", { default = -12, t = "dB" }),
	Parameter:new("pan", { default = 0, min = -1, max = 1, centered = true, fmt = "%0.2f" }),
}

deviceList.instruments = {}

deviceList.instruments.sine = {
	number = 0,
	parameters = {
		Parameter:new("feedback", { default = 1.0, max = 1.5 }),
	},
	mono = true,
}

deviceList.instruments.wavetable = {
	number = 1,
	parameters = {},
	mono = true,
}

deviceList.effects = {}

deviceList.effects.gain = {
	number = 0,
	parameters = {
		Parameter:new("gain", { default = 0, t = "dB" }),
	},
}

return deviceList
