local wav = require("./lib/wav")

local M = {}

local channelCount = 2
local sampleRate = 44100
local bitDepth = 16

local w

function M.open()
	w = wav.create_context("../out/audiotest.wav", "w")

	w.init(channelCount, sampleRate, bitDepth)
end

function M.append(samples)
	w.write_samples_interlaced(samples)
end

function M.close()
	w.finish()
end

return M
