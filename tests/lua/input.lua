-- Live note input: events for a held note reach the channel it started on.
-- cargo run -- --test-run tests/lua/input.lua

local midi = require("midi")
local note_input = require("note_input")

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

-- the audio backend isn't running, tokens only need to be unique
local next_token = 0
tessera.audio.get_token = function()
	next_token = next_token + 1
	return next_token
end

return h.run(tests)
