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

function audiolib.status()
	return stream_handle ~= nil
end

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
		end
	else
		print("Stream already running!")
	end
end

function audiolib.quit()
	if stream_handle then
		stream_handle = nil
		-- force garbage collection so the finalizer gets called immediately
		collectgarbage()
	end
end

function audiolib.send_CV(index, t)
	if stream_handle then
		lib.send_CV(stream_handle, index, t[1], t[2])
	end
end

function audiolib.send_noteOn(index, t)
	if stream_handle then
		lib.send_noteOn(stream_handle, index, t[1], t[2], 0) -- id will be used for polyphony
	end
end

function audiolib.send_pan(index, t)
	if stream_handle then
		lib.send_pan(stream_handle, index, t[1], t[2])
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

function audiolib.add_channel(instrument_index)
	if stream_handle then
		lib.add_channel(stream_handle, instrument_index)
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
