local log = require("log")
local midi = require("midi")

local RENDER_BLOCK_SIZE = 64

local engine = {}

engine.playing = false
engine.render_progress = 0
engine.render_end = 8

function engine.start()
	engine.seek(project.transport.start_time)
	engine.playing = true

	-- TODO: expose option to chase midi notes
	local chase = false

	for _, v in ipairs(ui_channels) do
		v.roll:start(chase)
	end
end

function engine.stop()
	engine.playing = false

	local added_notes = {}
	local total = 0

	for i, v in ipairs(ui_channels) do
		v.voice_alloc:allNotesOff()

		added_notes[i] = v.roll.recorded_notes
		total = total + #added_notes[i]
		v.roll:stop()
	end
	if total > 0 then
		local c = command.noteAdd.new(added_notes)
		command.register(c)
	end
end

function engine.seek(time)
	assert(not engine.playing)
	project.transport.time = time
end

function engine.update(dt)
	if engine.playing then
		project.transport.time = project.transport.time + dt
		if tessera.audio.ok() then
			for _, v in ipairs(ui_channels) do
				v.roll:playback()
			end
		end
	end

	engine.parseMessages()
end

function engine.renderStart()
	tessera.audio.flush()
	midi.flush()

	if not tessera.audio.ok() then
		log.error("Can't render, tessera.audio offline.")
		return
	end

	assert(audio_status == "running")
	audio_status = "render"

	engine.render_end = engine.endTime() + 2.0
	engine.render_progress = 0

	if engine.playing then
		engine.stop()
	end
	engine.start()

	log.info("Start render.")

	mouse:setCursor("wait")
	mouse:endFrame()

	tessera.audio.setRendering(true)

	-- sleep for a bit to make sure the audio thread is done
	-- TODO: find something better
	tessera.timer.sleep(0.01)
end

function engine.render()
	assert(tessera.audio.isRendering())

	local dt = RENDER_BLOCK_SIZE / tessera.audio.getSampleRate()

	-- Try to hit 16 ms to keep things responsive
	local target_ms = 16
	local t_start = tessera.timer.getTime()
	for i = 1, 3000 do
		local success = tessera.audio.renderBlock()
		if not success then
			log.error("Failed to render block.")
			engine.renderCancel()
			return
		end

		engine.update(dt)
		engine.render_progress = engine.render_progress + dt
		if engine.render_progress >= engine.render_end then
			log.info("Finished render.")
			tessera.audio.renderFinish()
			engine.renderEnd()
			break
		end

		local t_now = (tessera.timer.getTime() - t_start) * 1000
		if t_now > target_ms then
			print(tostring(i) .. " blocks rendered")
			break
		end
	end
end

function engine.renderEnd()
	midi.flush()

	tessera.audio.setRendering(false)
	mouse:setCursor("default")
	audio_status = "running"
	engine.stop()
end

-- update UI with messages from tessera.audio
function engine.parseMessages()
	while true do
		local p = tessera.audio.pop()
		if p == nil then
			return
		end
		if p.tag == "cpu" then
			workspace.cpu_load = p.cpu_load
		elseif p.tag == "meter" then
			workspace.meter.l = util.to_dB(p.l)
			workspace.meter.r = util.to_dB(p.r)
		end
	end
end

function engine.endTime()
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

return engine
