local VoiceAlloc = {}

VoiceAlloc.__index = VoiceAlloc

function VoiceAlloc.new(channel_index)
	local self = setmetatable({}, VoiceAlloc)

	self.channel_index = channel_index

	return self
end

function VoiceAlloc.noteOn()
	error("todo")
end

function VoiceAlloc.noteOff()
	error("todo")
end

return VoiceAlloc
