local polyphony = 16
-- local n_index = 1
local tracks = {}

function newTrack()
	local new = {}
	new.channel = 0
	new.note = 0
	new.offset = 0
	new.vel = 0
	new.age = 0
	new.isPlaying = false
	return new
end

for i = 1, polyphony do
	tracks[i] = newTrack()
end

-- polyphonic note stealing logic
-- doesnt work well for mono
function handle_midi(event)
	local index = -1

	for i, ch in ipairs(channelHandler.list) do
		if ch.armed then
			index = ch.index
		end
	end

	if index ~= -1 then
		if event.name == "note on" then
			print(event.note)
			local oldestPlaying_age, oldestPlaying_i = -1, -1
			local oldestReleased_age, oldestReleased_i = -1, -1
			for j, b in ipairs(tracks) do
				b.age = b.age + 1
				if b.isPlaying then
					-- track note is on
					if b.age > oldestPlaying_age then
						oldestPlaying_i = j
						oldestPlaying_age = b.age
					end
				else
					-- track is free
					if b.age > oldestReleased_age then
						oldestReleased_i = j
						oldestReleased_age = b.age
					end
				end
			end
			local new_i
			if oldestReleased_i ~= -1 then
				new_i = oldestReleased_i
			else
				new_i = oldestPlaying_i
			end
			tracks[new_i].channel = event.channel
			tracks[new_i].note = event.note
			tracks[new_i].age = 0
			tracks[new_i].isPlaying = true
			tracks[new_i].vel = event.vel

			audiolib.send_noteOn(index, tracks[new_i].note + tracks[new_i].offset, event.vel)
		end

		if event.name == "note off" then
			for _, b in ipairs(tracks) do
				if b.note == event.note then
					print("noteoff")
					b.isPlaying = false
					b.age = 0
					b.vel = 0
					audiolib.send_CV(index, b.note + b.offset, 0)
					-- b.note = -1

					break
				end
			end
		end

		if event.name == "CC" then
			for _, b in ipairs(tracks) do
				if b.channel == event.channel then
					if event.cc == 74 then
						b.vel = event.y
						audiolib.send_CV(index, b.note + b.offset, b.vel)
						break
					end
				end
			end
		end

		if event.name == "pitchbend" then
			for _, b in ipairs(tracks) do
				if b.channel == event.channel then
					b.offset = event.offset
					audiolib.send_CV(index, b.note + b.offset, b.vel)
					break
				end
			end
		end
	end
end
