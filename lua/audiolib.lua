-- @todo fix path
if release then
	package.cpath = package.cpath .. ";../target/release/?.dll"
else
	package.cpath = package.cpath .. ";../target/debug/?.dll"
end

local lib = require("rust_backend")
local stream_handle = nil

local audiolib = {}

audiolib.userdata = nil
audiolib.paused = true
audiolib.status = "wait"

function audiolib.load(host, device)
	if stream_handle == nil then
		host = host or "default"
		device = device or "default"

		audiolib.userdata = lib.stream_new(host, device)

		if audiolib.userdata == nil then
			print("Stream setup failed!")
			return
		else
			audiolib.paused = false
			audiolib.status = "running"
		end
	else
		print("Stream already running!")
	end
end

function audiolib.quit()
	if audiolib.userdata then
		audiolib.userdata = nil
		-- force GC to finalize the stream data
		collectgarbage()
		audiolib.status = "offline"
	end
end

function audiolib.send_cv(index, pitch, vel)
	if audiolib.userdata then
		audiolib.userdata:send_cv(index, pitch, vel)
	end
end

function audiolib.send_note_on(index, pitch, vel)
	if audiolib.userdata then
		audiolib.userdata:send_note_on(index, pitch, vel, 0) -- id will be used for polyphony
	end
end

function audiolib.send_pan(index, gain, pan)
	if audiolib.userdata then
		audiolib.userdata:send_pan(index, gain, pan)
	end
end

function audiolib.send_mute(index, mute)
	if audiolib.userdata then
		audiolib.userdata:send_mute(index, mute)
	end
end

function audiolib.send_param(ch_index, device_index, index, value)
	if audiolib.userdata then
		audiolib.userdata:send_param(ch_index, device_index, index, value)
	end
end

function audiolib.play()
	if audiolib.userdata then
		audiolib.userdata:play()
		audiolib.paused = false
	end
end

function audiolib.pause()
	if audiolib.userdata then
		audiolib.userdata:pause()
		audiolib.paused = true
	end
end

function audiolib.add_channel(instrument_number)
	if audiolib.userdata then
		audiolib.userdata:add_channel(instrument_number)
	end
end

function audiolib.add_effect(channel_index, effect_number)
	if audiolib.userdata then
		audiolib.userdata:add_effect(channel_index, effect_number)
	end
end

function audiolib.render_block()
	if audiolib.paused then
		if audiolib.userdata then
			local block = audiolib.userdata:render_block()

			return block
		end
	else
		print("pause stream before rendering!")
	end
end

function audiolib.get_spectrum()
	if audiolib.userdata then
		local block = audiolib.userdata:get_spectrum()

		return block
	end
end

function audiolib.parse_messages()
	if audiolib.userdata then
		while not audiolib.userdata:rx_is_empty() do
			local p = audiolib.userdata:rx_pop()
			if p.tag == "cpu" then
				workspace.cpu_load = p.cpu_load
			elseif p.tag == "meter" then
				workspace.meter.l = util.to_dB(p.l)
				workspace.meter.r = util.to_dB(p.r)
			end
		end
	end
end

return audiolib
