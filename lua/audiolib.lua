-- wrapper around rust ffi
-- respects GC bindings so theres no memory leaks

local ffi = require("ffi")
local cstring = require("cstring")

require("header")

local lib_path = "../target/debug"
if release then
	lib_path = "../target/release"
end

print(lib_path)

local lib = ffi.load(lib_path .. "/audiolib.dll")
local stream_handle = nil

local audiolib = {}

audiolib.paused = true

stream_handle__ = nil

function audiolib.load(host, device)
	if stream_handle == nil then
		host = host or "default"
		device = device or "default"
		stream_handle = ffi.gc(lib.stream_new(cstring(host), cstring(device)), lib.stream_free)
		-- stream_handle__ = lib.stream_new()

		audiolib.paused = false
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
		lib.send_CV(stream_handle, index,t[1], t[2] );
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

function audiolib.add()
	if stream_handle then
		lib.add(stream_handle)
	end
end

function audiolib.render_block()
	if audiolib.paused then
		if stream_handle then
			local block = lib.render_block(stream_handle);
			ffi.gc(block, lib.block_free)

			return block
		end
	else
		print("pause stream before rendering!!")
	end
end

return audiolib