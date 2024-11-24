local tuning = require("tuning")
local View = require("view")

local Song = View:derive("Song")

function Song:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	self.sx = 90
	self.sy = -12

	self.ox = 200
	self.oy = 900
	return new
end

function Song:proj_time(t)
	return t * self.sx + self.ox
end

function Song:proj_time_inv(t)
	return (t - self.ox) / self.sx
end

function Song:proj_pitch(p)
	return p * self.sy + self.oy
end

function Song:proj_pitch_inv(p)
	return (p - self.oy) / self.sy
end

function Song:invTransform(x, y)
	return (x - self.ox) / self.sx, (y - self.oy) / self.sy
end

function Song:draw()
	love.graphics.setColor(theme.bg_nested)
	love.graphics.rectangle("fill", 0, 0, self.w, self.h)

	-- draw grid
	love.graphics.setColor(theme.line)
	local ix, iy = self:invTransform(0, 0)
	local ex, ey = self:invTransform(self.w, self.h)

	-- octaves
	for i = math.floor(ey / 12) + 1, math.floor(iy / 12) do
		local py = self:proj_pitch(i * 12)
		love.graphics.line(0, py, self.w, py)
	end

	-- time grid
	for i = math.floor(ix / 2) + 1, math.floor(ex / 2) do
		local px = self:proj_time(i * 2)
		love.graphics.line(px, 0, px, self.h)
	end

	-- playhead
	local px = self:proj_time(project.transport.time)
	love.graphics.setColor(0.8, 0.2, 0.2)
	love.graphics.line(px, 0, px, self.h)

	-- draw notes
	love.graphics.setFont(resources.fonts.notes)
	for ch_index, ch in ipairs(project.channels) do
		if ch.visible then
			-- if selection.channel_index == ch_index then
			-- 	love.graphics.setColor(0.9, 0.9, 0.9)
			-- else
			-- 	love.graphics.setColor(0.4, 0.4, 0.4)
			-- end
			for _, note in ipairs(ch.notes) do
				local t_start = note.time
				local p_start = tuning.getPitch(note.pitch)
				local x0 = self:proj_time(t_start)
				local y0 = self:proj_pitch(p_start)

				love.graphics.setColor(0.6, 0.6, 0.6)
				love.graphics.line(x0, y0, x0, y0 - 24 * note.vel)
				love.graphics.line(x0 - 2, y0 - 24 * note.vel, x0 + 2, y0 - 24 * note.vel)
				-- love.graphics.setColor(theme.bg_nested)
				-- love.graphics.circle("fill", x0, y0 - 24 * note.vel, 3)
				-- love.graphics.setColor(0.6, 0.6, 0.6)
				-- love.graphics.circle("line", x0, y0 - 24 * note.vel, 3)

				for i = 1, #note.verts - 1 do
					local x1 = self:proj_time(t_start + note.verts[i][1])
					local x2 = self:proj_time(t_start + note.verts[i + 1][1])
					local y1 = self:proj_pitch(p_start + note.verts[i][2])
					local y2 = self:proj_pitch(p_start + note.verts[i + 1][2])
					local w1 = note.verts[i][3] * 12
					local w2 = note.verts[i + 1][3] * 12
					love.graphics.setColor(0.3, 0.3, 0.3)
					love.graphics.polygon("fill", x1, y1 + w1, x2, y2 + w2, x2, y2 - w2, x1, y1 - w1, x1, y1 + w1)
					love.graphics.setColor(0.9, 0.9, 0.9)
					love.graphics.line(x1, y1, x2, y2)
				end

				love.graphics.setColor(theme.bg_nested)
				love.graphics.circle("fill", x0, y0, 3)
				love.graphics.setColor(0.9, 0.9, 0.9)
				love.graphics.circle("line", x0, y0, 3)

				love.graphics.setColor(theme.ui_text)

				local note_name = tuning.getName(note.pitch)
				util.drawText(note_name, x0 + 6, y0 - 9, self.w, 0)
			end
		end
	end
end

function Song:keypressed(key)
	local zoom
	if key == "kp+" then
		zoom = 1.2
	elseif key == "kp-" then
		zoom = 1 / 1.2
	end

	if zoom then
		local mx, my = self:getMouse()
		self.sx = self.sx * zoom
		self.sy = self.sy * zoom

		self.ox = self.ox + (mx - self.ox) * (1 - zoom)
		self.oy = self.oy + (my - self.oy) * (1 - zoom)
		return true
	end
end

return Song
