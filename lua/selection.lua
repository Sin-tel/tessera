local selection = {}

selection.list = {}
selection.mask = {}

function selection.set(mask)
	selection.mask = mask
	selection.refresh()
end

function selection.subtract(mask)
	for k in pairs(mask) do
		selection.mask[k] = nil
	end
	selection.refresh()
end

function selection.add(mask)
	for k in pairs(mask) do
		selection.mask[k] = true
	end
	selection.refresh()
end

function selection.set_from_notes(notes)
	selection.mask = {}
	for _, v in pairs(notes) do
		for _, note in ipairs(v) do
			selection.mask[note] = true
		end
	end
	selection.refresh()
end

function selection.refresh()
	selection.list = {}
	for k in pairs(selection.mask) do
		table.insert(selection.list, k)
	end
end

function selection.is_empty()
	return #selection.list == 0
end

function selection.deselect()
	selection.list = {}
	selection.mask = {}
end

function selection.get_notes()
	-- get selected notes as a table per channel

	local notes = {}
	for ch_index, channel in ipairs(project.channels) do
		if channel.notes then
			notes[ch_index] = {}
			for _, note in ipairs(channel.notes) do
				if selection.mask[note] then
					table.insert(notes[ch_index], note)
				end
			end
		end
	end
	return notes
end

function selection.remove_inactive()
	for _, channel in ipairs(project.channels) do
		if channel.notes and (not channel.visible or channel.lock) then
			for _, v in ipairs(channel.notes) do
				selection.mask[v] = nil
			end
		end
	end
	selection.refresh()
end

function selection.select_default_channel()
	-- just select the first available channel that is not master
	local index = nil
	for i, channel in ipairs(project.channels) do
		if i > 1 and (not channel.visible or channel.lock) then
			index = i
			break
		end
	end
	selection.ch_index = index
	selection.device_index = nil
end

return selection
