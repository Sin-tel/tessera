local backend = require("backend")
local log = require("log")

local engine = {}

engine.playing = false

function engine.start()
	engine.playing = true
	engine.seek(project.transport.start_time)
end

function engine.stop()
	engine.playing = false
end

function engine.seek(time)
	project.transport.time = time
end

function engine.update(dt)
	if engine.playing then
		project.transport.time = project.transport.time + dt
	end

	if backend:running() then
		engine.parseMessages()
	end
end

function engine.render()
	--TODO: make this not block the UI

	if not backend:running() then
		log.error("Backend offline.")
		return
	end

	log.info("Start render.")

	mouse:setCursor("wait")
	mouse:endFrame()

	backend:setPaused(true)

	-- sleep for a bit to make sure the audio thread is done
	love.timer.sleep(0.01)

	for _ = 1, 5000 do
		local success = backend:renderBlock()
		if not success then
			log.error("Failed to render block.")
			backend:play()
			return
		end
		engine.parseMessages()
	end
	log.info("Finished render.")
	backend:renderFinish()

	backend:setPaused(false)

	mouse:setCursor("default")
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

return engine
