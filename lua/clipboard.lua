local clipboard = {}

clipboard.notes = {}

function clipboard.set(notes)
	clipboard.notes = util.clone(notes)
end

function clipboard.is_empty()
	local total = 0

	for _, v in ipairs(clipboard.notes) do
		total = total + #v
	end

	return total == 0
end

return clipboard
