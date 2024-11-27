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

	self.ox_ = self.ox
	self.oy_ = self.oy

	self.sx_ = self.sx
	self.sy_ = self.sy
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

function Song:update()
	if self.drag then
		self.ox = self.drag_ix + mouse.dx
		self.oy = self.drag_iy + mouse.dy
		-- should be instant
		self.ox_ = self.ox
		self.oy_ = self.oy
	end

	if mouse.scroll and self:focus() then
		local zoom = math.exp(0.15 * mouse.scroll)
		local mx, my = self:getMouse()
		self.sx_ = self.sx_ * zoom
		self.sy_ = self.sy_ * zoom

		self.ox_ = self.ox_ + (mx - self.ox_) * (1 - zoom)
		self.oy_ = self.oy_ + (my - self.oy_) * (1 - zoom)
	end
end

function Song:draw()
	love.graphics.setColor(theme.bg_nested)
	love.graphics.rectangle("fill", 0, 0, self.w, self.h)

	-- draw grid
	local ix, iy = self:invTransform(0, 0)
	local ex, ey = self:invTransform(self.w, self.h)

	local sf = 0.5
	self.ox = self.ox + sf * (self.ox_ - self.ox)
	self.oy = self.oy + sf * (self.oy_ - self.oy)
	self.sx = self.sx + sf * (self.sx_ - self.sx)
	self.sy = self.sy + sf * (self.sy_ - self.sy)

	-- pitch grid
	local oct = tuning.generators[1]
	for i = math.floor((ey - 60) / oct), math.floor((iy - 60) / oct) do
		if self.sy < -60 then
			love.graphics.setColor(theme.grid)
			for j, _ in ipairs(tuning.chromatic_table) do
				local py = self:proj_pitch(tuning.getPitch(tuning.fromMidi(j + 12 * i + 60)))
				love.graphics.line(0, py, self.w, py)
			end
		elseif self.sy < -20 then
			love.graphics.setColor(theme.grid)
			for j, _ in ipairs(tuning.diatonic_table) do
				local py = self:proj_pitch(tuning.getPitch(tuning.fromDiatonic(j, i)))
				love.graphics.line(0, py, self.w, py)
			end
		end
		love.graphics.setColor(theme.grid_highlight)
		local py = self:proj_pitch(tuning.getPitch({ i }))
		love.graphics.line(0, py, self.w, py)
	end

	-- time grid
	local grid_t_res = 4 ^ math.floor(3.5 - math.log(self.sx, 4))
	for i = math.floor(ix / grid_t_res) + 1, math.floor(ex / grid_t_res) do
		love.graphics.setColor(theme.grid)
		if i % 4 == 0 then
			love.graphics.setColor(theme.grid_highlight)
		end
		local px = self:proj_time(i * grid_t_res)
		love.graphics.line(px, 0, px, self.h)
	end

	-- playhead
	local px = self:proj_time(project.transport.time)
	love.graphics.setColor(theme.playhead)
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
				local vo = 32 * note.vel
				love.graphics.line(x0, y0, x0, y0 - vo)
				love.graphics.line(x0 - 2, y0 - vo, x0 + 2, y0 - vo)
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
				util.drawText(note_name, x0 + 5, y0 - 10, self.w, 0)
			end
		end
	end
end

function Song:keypressed(key)
	local zoom
	if key == "kp+" then
		zoom = math.sqrt(2)
	elseif key == "kp-" then
		zoom = 1 / math.sqrt(2)
	end

	if zoom then
		-- local mx, _ = self:getMouse()
		local mx = self.w * 0.25
		self.sx_ = self.sx_ * zoom
		self.ox_ = self.ox_ + (mx - self.ox_) * (1 - zoom)
		return true
	end
end

function Song:mousepressed()
	if mouse.button == 3 then
		self.drag = true
		-- local mx, my = self:getMouse()

		self.drag_ix = self.ox
		self.drag_iy = self.oy
	end
end

function Song:mousereleased()
	if mouse.button_released == 3 then
		self.drag = false
	end
end

return Song
