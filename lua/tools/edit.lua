local tuning = require("tuning")

local edit = {}

edit.ox = 0
edit.oy = 0

function edit:mousepressed(song)
	local mx, my = song:getMouse()

	self.ox = mx
	self.oy = my

	self.selection_active = true
end

function edit:mousedown(song)
	--
end

function edit:mousereleased(song)
	local mx, my = song:getMouse()

	self.selection_active = false

	selection.notes = {}

	local x1 = math.min(self.ox, mx)
	local y1 = math.min(self.oy, my)
	local x2 = math.max(self.ox, mx)
	local y2 = math.max(self.oy, my)

	-- TODO factor out selection stuff
	local ch_index = 1
	for i, v in ipairs(project.channels[ch_index].notes) do
		local t_start = v.time
		local p_start = tuning.getPitch(v.pitch)
		local x0 = song.transform:time(t_start)
		local y0 = song.transform:pitch(p_start)

		if x1 < x0 and x0 < x2 and y1 < y0 and y0 < y2 then
			selection.notes[v] = true
		end
	end
end

function edit:draw(song)
	local mx, my = song:getMouse()

	if self.selection_active then
		love.graphics.setColor(util.color_alpha(theme.selection, 0.02))
		love.graphics.rectangle("fill", self.ox, self.oy, mx - self.ox, my - self.oy)
		love.graphics.setColor(theme.selection)

		love.graphics.rectangle("line", self.ox + 0.5, self.oy + 0.5, mx - self.ox, my - self.oy)
	end
end

return edit
