local tuning = require("tuning")

local select_rect = {}

select_rect.ox = 0
select_rect.oy = 0

function select_rect:mousepressed(canvas)
	local mx, my = canvas:get_mouse()

	self.ox = mx
	self.oy = my

	self.active = true
end

function select_rect:mousedown(canvas) end

function select_rect:mousereleased(canvas)
	local mx, my = canvas:get_mouse()

	self.active = false

	local mask = {}

	local bx1 = math.min(self.ox, mx)
	local by1 = math.min(self.oy, my)
	local bx2 = math.max(self.ox, mx)
	local by2 = math.max(self.oy, my)

	for _, channel in ipairs(project.channels) do
		if channel.visible and not channel.lock then
			for _, note in ipairs(channel.notes) do
				-- broad phase: reject based on time range
				local t_start = note.time
				local t_end = note.time + note.verts[#note.verts][1]
				local x_start = canvas.transform:time(t_start)
				local x_end = canvas.transform:time(t_end)

				if x_end >= bx1 and x_start <= bx2 then
					-- narrow phase, do a line-box intersection for each segment
					local base_pitch = tuning.get_pitch(note.interval)

					local x1 = canvas.transform:time(t_start)
					local y1 = canvas.transform:pitch(base_pitch + note.verts[1][2])

					for k = 2, #note.verts do
						local vert = note.verts[k]

						local x2 = canvas.transform:time(t_start + vert[1])
						local y2 = canvas.transform:pitch(base_pitch + vert[2])

						if util.line_box_intersect(x1, y1, x2, y2, bx1, by1, bx2, by2) then
							-- hit
							mask[note] = true
							break
						end

						x1, y1 = x2, y2
					end
				end
			end
		end
	end

	if modifier_keys.ctrl then
		selection.subtract(mask)
	elseif modifier_keys.shift then
		selection.add(mask)
	else
		selection.set(mask)
	end
end

function select_rect:draw(canvas)
	local mx, my = canvas:get_mouse()

	if self.active then
		local c = theme.selection
		tessera.graphics.set_color_f(c[1], c[2], c[3], 0.16)
		tessera.graphics.rectangle("fill", self.ox, self.oy, mx - self.ox, my - self.oy)
		tessera.graphics.set_color(theme.selection)

		tessera.graphics.rectangle("line", self.ox + 0.5, self.oy + 0.5, mx - self.ox, my - self.oy)
	end
end

return select_rect
