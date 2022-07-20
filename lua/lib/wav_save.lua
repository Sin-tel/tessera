require("./lib/wav")

local M = {}

-- @todo set this when opening file
local channelCount = 2
local sampleRate = 44100
local bitDepth = 16 -- bits per sample

local w

function M.open()
	w = wav.create_context("audiotest.wav", "w")

	w.init(channelCount, sampleRate, bitDepth)
end

function M.append(samples)
	w.write_samples_interlaced(samples) -- ???
end

function M.close()
	w.finish()
end

return M
