local Roll = require("roll")
local log = require("log")
local tuning = require("tuning")

local Channel = {}

Channel.__index = Channel

function Channel.new(ch_index, data, instrument, widget)
	local self = setmetatable({}, Channel)

	self.ch_index = ch_index

	-- reference to project data
	self.data = data
	self.mute_old = false

	self.widget = widget
	self.instrument = instrument
	self.effects = {}

	self.roll = Roll.new(ch_index)

	return self
end

function Channel:event(event)
	self.roll:event(event)
	self:send_event(event)
end

-- send an event to the backend
function Channel:send_event(event)
	local token = event.token
	if event.name == "note_on" then
		local pitch = tuning.get_pitch(event.pitch)
		local v_curve = util.velocity_curve(event.vel)
		tessera.audio.note_on(self.ch_index, pitch, v_curve, token)
	elseif event.name == "note_off" then
		tessera.audio.note_off(self.ch_index, token)
	elseif event.name == "pitch" then
		tessera.audio.pitch(self.ch_index, event.offset, token)
	elseif event.name == "pressure" then
		tessera.audio.pressure(self.ch_index, event.pressure, token)
	elseif event.name == "sustain" then
		tessera.audio.sustain(self.ch_index, event.sustain)
	else
		log.warn("unhandled event: ", util.dump(event))
	end
end

function Channel:reset()
	self.mute_old = false
end

return Channel
