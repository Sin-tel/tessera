local log = require("log")
local midi = require("midi")

local RENDER_BLOCK_SIZE = 64

local engine = {}

engine.playing = false
engine.render_progress = 0
engine.render_end = 8
engine.time = 0
engine.frame_time = 0

engine.meter_id = nil
engine.meter_l = 0
engine.meter_r = 0

function engine.start()
	engine.seek(project.transport.start_time)
	engine.playing = true

	for _, v in ipairs(ui_channels) do
		v.roll:start(project.settings.chase)
	end
end

function engine.stop()
	if engine.playing == false then
		return
	end
	engine.playing = false

	local added_notes = {}
	local total = 0

	tessera.audio:all_notes_off()
	for i, v in ipairs(ui_channels) do
		added_notes[i] = v.roll.recorded_notes
		total = total + #added_notes[i]
		v.roll:stop()
	end
	if total > 0 then
		local c = command.NoteAdd.new(added_notes)
		command.register(c)
	end
end

function engine.seek(time)
	if engine.playing then
		log.warn("Engine was playing while seeking")
		return
	end
	engine.time = time
end

function engine.update(dt)
	if engine.playing then
		engine.time = engine.time + dt
		engine.frame_time = engine.frame_time + dt
		if tessera.audio.ok() then
			for _, v in ipairs(ui_channels) do
				v.roll:playback(v)
			end
		end
	end
	engine.parse_errors()
	engine.parse_messages()
end

function engine.update_frame()
	engine.frame_time = 0
end

function engine.render_start()
	engine.stop()
	tessera.audio.flush()
	midi.flush()

	if not tessera.audio.ok() then
		log.error("Can't render, audio offline.")
		return
	end

	assert(audio_status == "running")
	audio_status = "render"

	engine.render_end = engine.end_time() + 2.0
	engine.render_progress = project.transport.start_time

	if engine.playing then
		engine.stop()
	end
	engine.start()

	log.info("Start render.")

	mouse:set_cursor("wait")
	mouse:end_frame()

	tessera.audio.set_rendering(true)
	tessera.audio.clear_messages()
end

function engine.get_progress_relative()
	return (engine.render_progress - project.transport.start_time) / (engine.render_end - project.transport.start_time)
end

function engine.render()
	assert(tessera.audio.is_rendering())

	local dt = RENDER_BLOCK_SIZE / engine.sample_rate

	-- Try to hit 16 ms to keep things responsive
	local target_ms = 16
	local t_start = tessera.get_time()
	for i = 1, 3000 do
		local success = tessera.audio.render_block()
		if not success then
			log.error("Failed to render block.")
			engine.renderCancel()
			return
		end

		engine.update(dt)
		engine.render_progress = engine.render_progress + dt
		if engine.render_progress >= engine.render_end then
			log.info("Finished render.")
			tessera.audio.render_finish()
			engine.render_finish()
			break
		end

		local t_now = (tessera.get_time() - t_start) * 1000
		if t_now > target_ms then
			print(tostring(i) .. " blocks rendered")
			break
		end
	end
end

function engine.render_finish()
	midi.flush()

	tessera.audio.set_rendering(false)
	mouse:set_cursor("default")
	audio_status = "running"
	engine.stop()
end

function engine.setup_stream()
	local host = setup.host
	local device = setup.configs[host].device
	local buffer_size = setup.configs[host].buffer_size
	if device then
		engine.meter_id = tessera.audio.setup(host, device, buffer_size)
	else
		engine.meter_id = nil
		log.error("No device configured")
	end
end

function engine.rebuild_stream()
	engine.flush_messages()
	engine.buffer_size = nil
	engine.sample_rate = nil
	if tessera.audio.ok() then
		log.info("Rebuilding stream")
		local host = setup.host
		local device = setup.configs[host].device
		local buffer_size = setup.configs[host].buffer_size
		if device then
			tessera.audio.rebuild(host, device, buffer_size)
		else
			log.error("No device configured")
			tessera.audio.quit()
		end
	else
		audio_status = "request"
	end
end

-- update UI with messages from tessera.audio
function engine.parse_messages()
	while true do
		local msg = tessera.pop_message()
		if msg == nil then
			return
		end
		if msg.tag == "StreamSettings" then
			engine.buffer_size = msg.buffer_size
			engine.sample_rate = msg.sample_rate
		elseif msg.tag == "Log" then
			log.append(msg.level, msg.message)
		else
			log.warn("unhandled message:")
			log.warn(util.dump(msg))
		end
	end
end

function engine.parse_errors()
	while true do
		local msg = tessera.audio.pop_error()
		if msg == nil then
			return
		end
		if msg.tag == "ResetRequest" then
			engine.rebuild_stream()
		elseif msg.tag == "DeviceNotAvailable" then
			tessera.audio.quit()
		else
			log.warn("unhandled message:")
			log.warn(util.dump(msg))
		end
	end
end

function engine.flush_messages()
	-- TODO: probably better to do this on the backend just before we rebuild stream
	while true do
		local msg = tessera.pop_message()
		if msg == nil then
			break
		end
	end
	while true do
		local msg = tessera.audio.pop_error()
		if msg == nil then
			break
		end
		log.info("Ignored error message: " .. msg.tag)
	end
end

function engine.end_time()
	local t_end = 0.0
	for _, ch in ipairs(project.channels) do
		for _, v in ipairs(ch.notes) do
			local t = v.time + v.verts[#v.verts][1]
			if t > t_end then
				t_end = t
			end
		end
	end
	return t_end
end

local function send_channel_parameters(ch, ch_index)
	local mute = ch.data.mute
	if ch.mute_old ~= mute then
		tessera.audio.send_channel_mute(ch_index, mute)
		ch.mute_old = mute
	end

	local gain = ch.data.gain
	if ch.gain_old ~= gain then
		tessera.audio.send_channel_gain(ch_index, gain)
		ch.gain_old = gain
	end
end

local function send_device_mute(device, ch_index, device_index)
	local mute = device.data.mute
	if device.mute_old ~= mute then
		tessera.audio.send_device_mute(ch_index, device_index, mute)
		device.mute_old = mute
	end
end

function engine.update_meters()
	local meters = tessera.audio.get_meters()

	if not meters then
		engine.meter_l = 0
		engine.meter_r = 0

		for _, ch in ipairs(ui_channels) do
			ch.meter_l = 0
			ch.meter_r = 0
			ch.instrument.meter_l = 0
			ch.instrument.meter_r = 0
			for _, fx in ipairs(ch.effects) do
				fx.meter_l = 0
				fx.meter_r = 0
			end
		end
		return
	end

	if engine.meter_id then
		engine.meter_l = meters[engine.meter_id][1]
		engine.meter_r = meters[engine.meter_id][2]
	end

	for _, ch in ipairs(ui_channels) do
		local i = ch.meter_id
		ch.meter_l = meters[i][1]
		ch.meter_r = meters[i][2]

		local k = ch.instrument.meter_id
		ch.instrument.meter_l = meters[k][1]
		ch.instrument.meter_r = meters[k][2]
		for _, fx in ipairs(ch.effects) do
			i = fx.meter_id
			fx.meter_l = meters[i][1]
			fx.meter_r = meters[i][2]
		end
	end
end

local function to_float(x)
	if type(x) == "boolean" then
		return x and 1 or 0
	else
		return x
	end
end

function engine.send_parameters()
	for ch_index, ch in ipairs(ui_channels) do
		send_channel_parameters(ch, ch_index)

		send_device_mute(ch.instrument, ch_index, 0)

		for l in ipairs(ch.instrument.parameters) do
			local new_value = ch.instrument.state[l]
			local old_value = ch.instrument.state_old[l]
			if old_value ~= new_value then
				local value = new_value
				tessera.audio.send_parameter(ch_index, 0, l, to_float(value))
				ch.instrument.state_old[l] = new_value
			end
		end

		for fx_index, fx in ipairs(ch.effects) do
			send_device_mute(fx, ch_index, fx_index)

			for l in ipairs(fx.parameters) do
				local new_value = fx.state[l]
				local old_value = fx.state_old[l]
				if old_value ~= new_value then
					local value = new_value
					tessera.audio.send_parameter(ch_index, fx_index, l, to_float(value))
					fx.state_old[l] = new_value
				end
			end
		end
	end
end

function engine.reset_parameters()
	for _, ch in ipairs(ui_channels) do
		ch:reset()
		ch.instrument:reset()
		for _, fx in ipairs(ch.effects) do
			fx:reset()
		end
	end
end

return engine
