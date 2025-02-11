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

return selection
