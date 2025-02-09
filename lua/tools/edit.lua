local edit = {}

-- edit.points = {}

edit.x = 0
edit.y = 0

function edit:mousepressed()
	-- edit.points = {}

	-- edit.x = mouseX
	-- edit.y = mouseY

	-- local pt = { edit.x, edit.y, pres }
	-- table.insert(edit.points, pt)
	-- edit.lastpoint = pt
end

function edit:mousedown()
	-- local dx = mouseX - edit.x
	-- local dy = mouseY - edit.y

	-- local l = math.sqrt(dx ^ 2 + dy ^ 2)
	-- if l > edit.radius then
	-- 	edit.x = mouseX - (edit.radius * dx / l)
	-- 	edit.y = mouseY - (edit.radius * dy / l)
	-- end

	-- if not (edit.lastpoint[1] == edit.x and edit.lastpoint[2] == edit.y) then
	-- 	edit.removePoints()
	-- 	local pt = { edit.x, edit.y, pres }
	-- 	table.insert(edit.points, pt)
	-- 	table.sort(edit.points, function(a, b)
	-- 		return a[1] < b[1]
	-- 	end)
	-- 	edit.lastpoint = pt
	-- end
end

function edit:mousereleased()
	-- for i, v in ipairs(edit.points) do
	-- 	v[1], v[2] = View.invTransform(v[1], v[2])
	-- end
	-- if #edit.points > 2 then
	-- 	edit.keep = {}
	-- 	for i in ipairs(edit.points) do
	-- 		edit.keep[i] = false
	-- 	end
	-- 	edit.keep[1] = true
	-- 	edit.keep[#edit.points] = true

	-- 	edit.simplify(1, #edit.points, true)

	-- 	local newTable = {}
	-- 	for i, v in ipairs(edit.points) do
	-- 		if edit.keep[i] then
	-- 			table.insert(newTable, v)
	-- 		end
	-- 	end
	-- 	edit.points = newTable

	-- 	table.sort(edit.points, function(a, b)
	-- 		return a[1] < b[1]
	-- 	end)

	-- 	Edit.addNote(edit.points)
	-- end
	-- edit.points = {}

	-- Edit.resampleAll()
end

function edit:draw()
	if self.selection_active then
		--
	end
end

return edit
