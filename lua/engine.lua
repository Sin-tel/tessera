local log = require("log")
local backend = require("backend")
local midi = require("midi")

local RENDER_BLOCK_SIZE = 64

local engine = {}

engine.playing = false
engine.render_progress = 0
engine.render_end = 8

function engine.start()
	engine.playing = true
	engine.seek(project.transport.start_time)

	for _, v in ipairs(ui_channels) do
		v.roll:start()
	end
end

function engine.stop()
	engine.playing = false

	for _, v in ipairs(ui_channels) do
		v.voice_alloc:allNotesOff()
		v.roll:stop()
	end
end

function engine.seek(time)
	project.transport.time = time

	if engine.playing then
		for _, v in ipairs(ui_channels) do
			v.voice_alloc:allNotesOff()
			v.roll:seek()
		end
	end
end

function engine.update(dt)
	if engine.playing then
		project.transport.time = project.transport.time + dt
		if backend:ok() then
			for _, v in ipairs(ui_channels) do
				v.roll:playback()
			end
		end
	end

	engine.parseMessages()
end

function engine.renderStart()
	backend:flush()
	midi.flush()

	if not backend:ok() then
		log.error("Can't render, backend offline.")
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

	backend:setRendering(true)

	-- sleep for a bit to make sure the audio thread is done
	-- TODO: find something better
	love.timer.sleep(0.01)
end

function engine.render()
	assert(backend:isRendering())

	local dt = RENDER_BLOCK_SIZE / backend:getSampleRate()

	local target_ms = 2
	local t_start = love.timer.getTime()
	for i = 1, 300 do
		local success = backend:renderBlock()
		if not success then
			log.error("Failed to render block.")
			engine.renderCancel()
			return
		end

		engine.update(dt)
		engine.render_progress = engine.render_progress + dt
		if engine.render_progress >= engine.render_end then
			log.info("Finished render.")
			backend:renderFinish()
			engine.renderEnd()
			break
		end

		local t_now = (love.timer.getTime() - t_start) * 1000
		if t_now > target_ms then
			print(i)
			break
		end
	end
end

function engine.renderEnd()
	midi.flush()

	backend:setRendering(false)
	mouse:setCursor("default")
	audio_status = "running"
	engine.stop()
end

-- update UI with messages from backend
function engine.parseMessages()
	while true do
		local p = backend:pop()
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
