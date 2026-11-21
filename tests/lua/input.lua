-- Live note input: events for a held note reach the channel it started on.
-- cargo run -- --test-run tests/lua/input.lua

local engine = require("engine")
local midi = require("midi")
local note_input = require("note_input")
local tuning = require("tuning")

local h = dofile("tests/lua/lib/harness.lua")

-- channels that record what they receive
local function stub_channels(n)
	local channels = {}
	for i = 1, n do
		channels[i] = {
			events = {},
			event = function(self, event)
				table.insert(self.events, event)
			end,
		}
	end
	return channels
end

local function names(channel)
	local list = {}
	for _, e in ipairs(channel.events) do
		table.insert(list, e.name)
	end
	return table.concat(list, " ")
end

local tests = {}

tests["midi note off after re-arming"] = function()
	local a, b = unpack(stub_channels(2))
	local device = midi.MidiDevice.new({ name = "test", mpe = false })
	device:event({ a }, { name = "note_on", channel = 0, note = 60, vel = 0.7 })
	-- arm switched to b
	device:event({ b }, { name = "pitchbend", channel = 0, pitchbend = 0.5 })
	device:event({ b }, { name = "note_off", channel = 0, note = 60 })
	assert(names(a) == "note_on pitch note_off", "a got: " .. names(a))
	assert(names(b) == "", "b got: " .. names(b))
	assert(a.events[3].token == a.events[1].token)
end

tests["midi repeated note on ends the old note"] = function()
	local a = unpack(stub_channels(1))
	local device = midi.MidiDevice.new({ name = "test", mpe = false })
	device:event({ a }, { name = "note_on", channel = 0, note = 60, vel = 0.7 })
	device:event({ a }, { name = "note_on", channel = 0, note = 60, vel = 0.7 })
	device:event({ a }, { name = "note_off", channel = 0, note = 60 })
	assert(names(a) == "note_on note_off note_on note_off", "a got: " .. names(a))
	assert(a.events[2].token == a.events[1].token)
	assert(a.events[4].token == a.events[3].token)
end

tests["midi sustain released after re-arming"] = function()
	local a, b = unpack(stub_channels(2))
	local device = midi.MidiDevice.new({ name = "test", mpe = false })
	device:event({ a }, { name = "controller", controller = 64, value = 127 })
	device:event({ b }, { name = "controller", controller = 64, value = 0 })
	assert(names(a) == "sustain sustain" and a.events[2].sustain == false, "a got: " .. names(a))
	assert(names(b) == "sustain" and b.events[1].sustain == false, "b got: " .. names(b))
end

tests["keyboard note off after selecting another channel"] = function()
	ui_channels = stub_channels(2)
	selection.ch_index = 1
	assert(note_input:keypressed("q"))
	selection.ch_index = 2
	assert(note_input:keyreleased("q"))
	assert(names(ui_channels[1]) == "note_on note_off", "1 got: " .. names(ui_channels[1]))
	assert(names(ui_channels[2]) == "", "2 got: " .. names(ui_channels[2]))
	assert(ui_channels[1].events[1].vel == note_input.DEFAULT_VELOCITY)
end

-- real channels, backend calls recorded as "name ch_index"
local function real_channels(n)
	local Channel = require("channel")
	local calls = {}
	for _, name in ipairs({ "note_on", "note_off", "pitch", "pressure", "sustain", "remove_channel" }) do
		tessera.audio[name] = function(ch_index)
			table.insert(calls, name .. " " .. ch_index)
		end
	end
	ui_channels = {}
	for i = 1, n do
		project.channels[i] = project.channels[i] or { notes = {}, control = {}, visible = true }
		project.channels[i].instrument = { name = "test" }
		ui_channels[i] = Channel.new(i, project.channels[i], {}, 0)
	end
	return calls
end

tests["keyboard note off after deleting its channel"] = function()
	local calls = real_channels(3)
	selection.ch_index = 3
	assert(note_input:keypressed("q"))
	command.run_and_register(command.RemoveChannel.new(3))
	assert(note_input:keyreleased("q"))
	local got = table.concat(calls, ", ")
	assert(got == "note_on 3, remove_channel 3", "backend got: " .. got)
end

tests["midi note off after deleting a channel before it"] = function()
	local calls = real_channels(3)
	local device = midi.MidiDevice.new({ name = "test", mpe = false })
	device:event({ ui_channels[2] }, { name = "note_on", channel = 0, note = 60, vel = 0.7 })
	device:event({ ui_channels[2] }, { name = "controller", controller = 64, value = 127 })
	-- channel 3 becomes 2
	command.run_and_register(command.RemoveChannel.new(2))
	device:event({ ui_channels[2] }, { name = "note_off", channel = 0, note = 60 })
	device:event({ ui_channels[2] }, { name = "controller", controller = 64, value = 0 })
	local got = table.concat(calls, ", ")
	assert(got == "note_on 2, sustain 2, remove_channel 2, sustain 2", "backend got: " .. got)
end

-- starting pressures of the note ons sent to the backend
local function record_pressure()
	local pressures = {}
	tessera.audio.note_on = function(_, _, _, _, pressure)
		table.insert(pressures, pressure)
	end
	return pressures
end

tests["note on sends a pressure"] = function()
	real_channels(2)
	local pressures = record_pressure()
	local interval = tuning.new_interval()
	ui_channels[2]:event({ name = "note_on", token = 1, interval = interval, vel = 0.5, offset = 0 })
	ui_channels[2]:event({ name = "note_on", token = 2, interval = interval, vel = 0.5, offset = 0, pressure = 0.2 })
	assert(pressures[1] == note_input.DEFAULT_PRESSURE and pressures[2] == 0.2, "got " .. util.dump(pressures))
end

tests["mpe pressure before the note on"] = function()
	local a = unpack(stub_channels(1))
	local mpe = midi.MidiDevice.new({ name = "mpe", mpe = true })
	mpe:event({ a }, { name = "pressure", channel = 2, pressure = 0.3 })
	mpe:event({ a }, { name = "note_on", channel = 2, note = 60, vel = 0.7 })
	mpe:event({ a }, { name = "note_on", channel = 3, note = 62, vel = 0.7 })
	assert(names(a) == "note_on note_on", "a got: " .. names(a))
	assert(a.events[1].pressure == 0.3, "pressure before the note on not used")
	assert(a.events[2].pressure == 0, "MPE note without pressure got " .. tostring(a.events[2].pressure))

	local b = unpack(stub_channels(1))
	local keyboard = midi.MidiDevice.new({ name = "keyboard", mpe = false })
	keyboard:event({ b }, { name = "pressure", channel = 0, pressure = 0.3 })
	keyboard:event({ b }, { name = "note_on", channel = 0, note = 60, vel = 0.7 })
	assert(b.events[1].pressure == nil, "non-MPE note should use the default")
end

-- Replays a recording made with examples/midi_dump.rs (in tests/lua/midi/) through `device`.
local function replay(device, sink, file)
	for line in io.lines("tests/lua/midi/" .. file) do
		local hex = line:match("^%s*[%d%.]+%s+(%x%x[ %x]-)%s+%[")
		if hex then
			local bytes = {}
			for b in hex:gmatch("%x%x") do
				table.insert(bytes, tonumber(b, 16))
			end
			local event = tessera.midi.parse(bytes)
			if event then
				device:event(sink, event)
			end
		end
	end
end

local function events_named(channel, name)
	local list = {}
	for _, e in ipairs(channel.events) do
		if e.name == name then
			table.insert(list, e)
		end
	end
	return list
end

-- Seaboard sends pressure only after the note on, the previous note's pressure must not carry over.
tests["seaboard: repeated notes on a channel start from zero pressure"] = function()
	local a = unpack(stub_channels(1))
	replay(midi.MidiDevice.new({ name = "seaboard", mpe = true }), { a }, "seaboard_taps.txt")
	local ons = events_named(a, "note_on")
	assert(#ons == 2, "expected 2 note ons, got " .. #ons)
	for i, e in ipairs(ons) do
		assert(e.pressure == 0, ("note %d started at pressure %s"):format(i, tostring(e.pressure)))
		assert(e.offset == 0, ("note %d started at pitch offset %s"):format(i, tostring(e.offset)))
	end
	assert(#events_named(a, "note_off") == 2)
end

tests["seaboard: overlapping notes keep their own pressure"] = function()
	local a = unpack(stub_channels(1))
	replay(midi.MidiDevice.new({ name = "seaboard", mpe = true }), { a }, "seaboard_overlap.txt")
	local ons = events_named(a, "note_on")
	assert(#ons == 2 and ons[2].pressure == 0, "second note did not start at 0")
	for _, e in ipairs(events_named(a, "pressure")) do
		-- 0x18 is the first pressure of the second note
		if e.pressure == 0x18 / 127 then
			assert(e.token == ons[2].token, "pressure went to the wrong note")
		end
	end
end

-- runs `f` with the transport recording, cleans up even if it fails
local function recording(f)
	project.transport = { recording = true }
	engine.playing = true
	local ok, err = pcall(f)
	engine.playing = false
	command.end_edit(engine)
	assert(ok, err)
end

tests["recorded notes keep their pressure"] = function()
	real_channels(2)
	local ch = ui_channels[2]
	local interval = tuning.new_interval()
	recording(function()
		engine.time = 1
		ch:event({ name = "note_on", token = 1, interval = interval, vel = 0.5, offset = 0 })
		ch:event({ name = "note_on", token = 2, interval = interval, vel = 0.5, offset = 0, pressure = 0.2 })
		engine.time = 2
		ch:event({ name = "pressure", token = 1, pressure = 0.8 })
		engine.time = 3
		ch:event({ name = "note_off", token = 1 })
		ch:event({ name = "note_off", token = 2 })
	end)

	local a, b = unpack(project.channels[2].notes)
	assert(
		util.deep_eq(a.verts, { { 0, 0, note_input.DEFAULT_PRESSURE }, { 1, 0, 0.8 }, { 2, 0, 0.8 } }),
		util.dump(a.verts)
	)
	assert(util.deep_eq(b.verts, { { 0, 0, 0.2 }, { 2, 0, 0.2 } }), util.dump(b.verts))
end

tests["playback sends the note's pressure"] = function()
	real_channels(2)
	local pressures = record_pressure()
	project.channels[2].notes = {
		{ interval = tuning.new_interval(), time = 1, vel = 0.5, verts = { { 0, 0, 0.3 }, { 1, 0, 0.3 } } },
	}
	local ch = ui_channels[2]
	engine.time = 0
	ch.roll:start(false)
	engine.time = 1.001
	ch.roll:playback(ch)
	assert(pressures[1] == 0.3, "got " .. util.dump(pressures))
end

-- the audio backend isn't running, tokens only need to be unique
local next_token = 0
tessera.audio.get_token = function()
	next_token = next_token + 1
	return next_token
end

return h.run(tests)
