local tuning = require("tuning")

local select_rect = {}

select_rect.ox = 0
select_rect.oy = 0

function select_rect:mousepressed(song)
	local mx, my = song:getMouse()

	self.ox = mx
	self.oy = my

	self.active = true
end

function select_rect:mousedown(song)
	--
end

function select_rect:mousereleased(song)
	local mx, my = song:getMouse()

	self.active = false

	local mask = {}

	local x1 = math.min(self.ox, mx)
	local y1 = math.min(self.oy, my)
	local x2 = math.max(self.ox, mx)
	local y2 = math.max(self.oy, my)

	-- TODO: fix ch_index
	local ch_index = 1
	for i, v in ipairs(project.channels[ch_index].notes) do
		local t_start = v.time
		local p_start = tuning.getPitch(v.pitch)
		local x0 = song.transform:time(t_start)
		local y0 = song.transform:pitch(p_start)

		if x1 < x0 and x0 < x2 and y1 < y0 and y0 < y2 then
			mask[v] = true
		end
	end

	selection.set(mask)
end

function select_rect:draw(song)
	local mx, my = song:getMouse()

	if self.active then
		love.graphics.setColor(util.color_alpha(theme.selection, 0.02))
		love.graphics.rectangle("fill", self.ox, self.oy, mx - self.ox, my - self.oy)
		love.graphics.setColor(theme.selection)

		love.graphics.rectangle("line", self.ox + 0.5, self.oy + 0.5, mx - self.ox, my - self.oy)
	end
end

return select_rect
