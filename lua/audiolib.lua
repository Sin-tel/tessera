-- wrapper around rust ffi
-- respects GC bindings so theres no memory leaks
-- should also handle nullpointers gracefully

local ffi = require("ffi")
local cstring = require("lib/cstring")

require("header")

-- should fix this
-- local lib_path = love.filesystem.getSource( ) .. "/../target/debug"
local lib_path = "../target/debug"
if release then
	lib_path = "../target/release"
end

-- print(lib_path)

local lib = ffi.load(lib_path .. "/audiolib.dll")
local stream_handle = nil

local audiolib = {}

audiolib.paused = true
audiolib.status = "wait"

function audiolib.load(host, device)
	if stream_handle == nil then
		host = host or "default"
		device = device or "default"

		stream_handle = lib.stream_new(cstring(host), cstring(device))

		-- check for null ptr
		if stream_handle == nil then
			print("Stream setup failed!")
			stream_handle = nil -- actually set it to nil instead of Cdata<NULL>
			return
		else
			stream_handle = ffi.gc(stream_handle, lib.stream_free)
			audiolib.paused = false
			audiolib.status = "running"
		end
	else
		print("Stream already running!")
	end
end

function audiolib.quit()
	if stream_handle then
		stream_handle = nil
		audiolib.status = "offline"
		-- force garbage collection so the finalizer gets called immediately
		collectgarbage()
	end
end

function audiolib.send_CV(index, pitch, vel)
	if stream_handle then
		lib.send_CV(stream_handle, index, pitch, vel)
	end
end

function audiolib.send_noteOn(index, pitch, vel)
	if stream_handle then
		lib.send_noteOn(stream_handle, index, pitch, vel, 0) -- id will be used for polyphony
	end
end

function audiolib.send_pan(index, gain, pan)
	if stream_handle then
		lib.send_pan(stream_handle, index, gain, pan)
	end
end

function audiolib.send_mute(index, mute)
	if stream_handle then
		lib.send_mute(stream_handle, index, mute)
	end
end

function audiolib.send_param(ch_index, device_index, index, value)
	if stream_handle then
		lib.send_param(stream_handle, ch_index, device_index, index, value)
	end
end

function audiolib.play()
	if stream_handle then
		lib.play(stream_handle)
		audiolib.paused = false
	end
end

function audiolib.pause()
	if stream_handle then
		lib.pause(stream_handle)
		audiolib.paused = true
	end
end

function audiolib.add_channel(instrument_number)
	if stream_handle then
		lib.add_channel(stream_handle, instrument_number)
	end
end

function audiolib.add_effect(channel_index, effect_number)
	if stream_handle then
		lib.add_channel(stream_handle, channel_index, effect_number)
	end
end

function audiolib.render_block()
	if audiolib.paused then
		if stream_handle then
			local block = lib.render_block(stream_handle)
			ffi.gc(block, lib.block_free)

			return block
		end
	else
		print("pause stream before rendering!")
	end
end

function audiolib.get_spectrum()
	if stream_handle then
		local block = lib.get_spectrum(stream_handle)
		ffi.gc(block, lib.block_free)

		-- Check for null ptr
		if block == nil or tonumber(block.len) == 0 then
			-- print("Failed to get spectrum!")
			return
		end

		-- Copy the block so we dont keep rust vecs around
		local s = block.ptr
		local spectrum = {}
		for i = 1, tonumber(block.len) do
			spectrum[i] = s[i - 1]
		end

		return spectrum
	end
end

function audiolib.parse_messages()
	if stream_handle then
		while not lib.rx_is_empty(stream_handle) do
			local p = lib.rx_pop(stream_handle)
			if p.tag == "Cpu" then
				workspace.cpu_load = p.cpu
			elseif p.tag == "Meter" then
				workspace.meter.l = to_dB(p.meter._0)
				workspace.meter.r = to_dB(p.meter._1)
			end
		end
	end
end

return audiolib
