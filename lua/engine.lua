local backend = require("backend")
local log = require("log")
local tuning = require("tuning")

local RENDER_BLOCK_SIZE = 64

local engine = {}

engine.playing = false
engine.render_progress = 0
engine.render_total = 8

local n_index = 1
local note_table = {}
local voices = {}

function engine.start()
	engine.playing = true
	engine.seek(project.transport.start_time)
	note_table = {}
	voices = {}

	--- TODO only playing back on channel 1 for now
	for i, v in ipairs(project.channels[1].notes) do
		assert(v.verts[1][1] == 0)
		table.insert(note_table, v)
	end
	table.sort(note_table, function(a, b)
		return a.time < b.time
	end)

	-- seek
	n_index = 1

	while note_table[n_index] and project.transport.time > note_table[n_index].time do
		n_index = n_index + 1
	end
end

function engine.stop()
	engine.playing = false

	for i = #voices, 1, -1 do
		local v = voices[i]
		local ch_index = 1 -- FIXME
		local note_i = v.n_index -- FIXME
		-- note off
		backend:noteOff(ch_index, note_i)
		table.remove(voices, i)
	end
end

function engine.seek(time)
	project.transport.time = time
end

function engine.update(dt)
	if engine.playing then
		project.transport.time = project.transport.time + dt
		if backend:ok() then
			engine.playback()
		end
	end

	engine.parseMessages()
end

function engine.playback()
	while note_table[n_index] and project.transport.time > note_table[n_index].time do
		local note = note_table[n_index]
		table.insert(voices, { n_index = n_index, v_index = 1 })

		-- note on
		local p = tuning.getPitch(note.pitch)
		local vel = util.velocity_curve(note.vel)

		local ch_index = 1 -- FIXME
		local note_i = n_index -- FIXME
		backend:noteOn(ch_index, p, vel, note_i)

		n_index = n_index + 1
	end

	for i = #voices, 1, -1 do
		local v = voices[i]
		local note = note_table[v.n_index]

		while v.v_index + 1 <= #note.verts and project.transport.time > note.time + note.verts[v.v_index + 1][1] do
			v.v_index = v.v_index + 1
		end

		local ch_index = 1 -- FIXME
		local note_i = v.n_index -- FIXME
		-- note off
		if v.v_index >= #note.verts then
			backend:noteOff(ch_index, note_i)
			table.remove(voices, i)
		else
			local p = tuning.getPitch(note.pitch)

			local t0 = note.verts[v.v_index][1]
			local t1 = note.verts[v.v_index + 1][1]
			local alpha = (project.transport.time - (note.time + t0)) / (t1 - t0)

			local p_off = util.lerp(note.verts[v.v_index][2], note.verts[v.v_index + 1][2], alpha)
			local press = util.lerp(note.verts[v.v_index][3], note.verts[v.v_index + 1][3], alpha)

			backend:sendCv(ch_index, p + p_off, press, note_i)
		end
	end
end

function engine.renderStart()
	-- TODO: flush midi and audio buffers

	if not backend:ok() then
		log.error("Can't render, backend offline.")
		return
	end

	assert(audio_status == "running")
	audio_status = "render"

	-- TODO: calculate how long we should render
	engine.render_total = 8
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

	local target_ms = 2 -- can probably set this higher
	local start = love.timer.getTime()
	for i = 1, 100 do
		local success = backend:renderBlock()
		if not success then
			log.error("Failed to render block.")
			engine.renderCancel()
			return
		end

		engine.update(dt)
		engine.render_progress = engine.render_progress + dt
		if engine.render_progress >= engine.render_total then
			log.info("Finished render.")
			backend:renderFinish()
			engine.renderEnd()
			break
		end

		local t_now = (love.timer.getTime() - start) * 1000
		if t_now > target_ms then
			break
		end
	end
end

function engine.renderEnd()
	-- TODO: flush midi and audio buffers

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

return engine
