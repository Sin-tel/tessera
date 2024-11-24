local backend = require("backend")
local log = require("log")
local tuning = require("tuning")

local engine = {}

engine.playing = false

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
end

function engine.seek(time)
	project.transport.time = time
end

function engine.update(dt)
	if engine.playing then
		project.transport.time = project.transport.time + dt
		if backend:running() then
			engine.playback()
		end
	end

	if backend:running() then
		engine.parseMessages()
	end
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
		print(note_i)
		backend:noteOn(ch_index, p, vel, note_i)

		n_index = n_index + 1
	end

	for i = #voices, 1, -1 do
		local v = voices[i]
		local note = note_table[v.n_index]

		while v.v_index < #note.verts and project.transport.time > note.time + note.verts[v.v_index + 1][2] do
			v.v_index = v.v_index + 1
		end

		-- note off
		if v.v_index >= #note.verts then
			local ch_index = 1 -- FIXME
			local note_i = v.n_index -- FIXME

			backend:noteOff(ch_index, note_i)
			table.remove(voices, i)
		end
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
