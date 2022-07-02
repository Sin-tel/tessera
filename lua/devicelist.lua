devicelist = {}

devicelist.channel = {
	Parameter:new("gain", {default = -12,  t = "dB"}),
	Parameter:new("pan", {default = 0, min = -1,max = 1, centered = true, fmt = "%0.2f"}),
}

devicelist.instruments = {}

devicelist.instruments.sine = {
	index = 0,
	parameters = {
		Parameter:new("feedback", {default = 1.0, max = 1.5}),
	},
}

devicelist.effects = {}
