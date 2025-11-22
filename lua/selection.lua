local selection = {}

selection.list = {}
selection.mask = {}

function selection.set(mask)
	if modifier_keys.ctrl then
		for k in pairs(mask) do
			selection.mask[k] = nil
		end
	else
		if modifier_keys.shift then
			for k in pairs(mask) do
				selection.mask[k] = true
			end
		else
			selection.mask = mask
		end
	end

	selection.refresh()
end

function selection.setNormal(mask)
	selection.mask = mask
	selection.refresh()
end

function selection.refresh()
	selection.list = {}
	for k in pairs(selection.mask) do
		table.insert(selection.list, k)
	end
end

function selection.isEmpty()
	return #selection.list == 0
end

function selection.deselect()
	selection.list = {}
	selection.mask = {}
end

function selection.getNotes()
	-- get selected notes as a table per channel

	local notes = {}
	for ch_index, channel in ipairs(project.channels) do
		notes[ch_index] = {}
		for _, note in ipairs(channel.notes) do
			if selection.mask[note] then
				table.insert(notes[ch_index], note)
			end
		end
	end
	return notes
end

function selection.removeChannel(ch)
	for _, v in ipairs(ch.notes) do
		selection.mask[v] = nil
	end
	selection.refresh()
end

return selection
