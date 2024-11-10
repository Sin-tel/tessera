local tuning = require("tuning")
local View = require("view")

local Song = View:derive("Song")

local function proj_time(t, w)
	return (t + 1) * 0.1 * w + 0.5
	-- return (t + 1) * 90 + 0.5
end

local function proj_pitch(p, h)
	return (0.5 - (p - 60) * 0.03) * h
end

function Song:draw()
	local w, h = self:getDimensions()
	-- local mx, my = self:getMouse()

	love.graphics.setColor(theme.bg_nested)
	love.graphics.rectangle("fill", 0, 0, w, h)

	local px = proj_time(project.transport.time, w)
	love.graphics.setColor(theme.line)
	love.graphics.line(px, 0, px, h)

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
				local x0 = proj_time(t_start, w)
				local y0 = proj_pitch(p_start, h)

				love.graphics.setColor(0.6, 0.6, 0.6)
				love.graphics.line(x0, y0, x0, y0 - 24 * note.vel)
				love.graphics.line(x0 - 2, y0 - 24 * note.vel, x0 + 2, y0 - 24 * note.vel)
				-- love.graphics.setColor(theme.bg_nested)
				-- love.graphics.circle("fill", x0, y0 - 24 * note.vel, 3)
				-- love.graphics.setColor(0.6, 0.6, 0.6)
				-- love.graphics.circle("line", x0, y0 - 24 * note.vel, 3)

				love.graphics.setColor(0.9, 0.9, 0.9)
				for i = 1, #note.verts - 1 do
					local x1 = proj_time(t_start + note.verts[i][2], w)
					local x2 = proj_time(t_start + note.verts[i + 1][2], w)
					local y1 = proj_pitch(p_start + note.verts[i][1], h)
					local y2 = proj_pitch(p_start + note.verts[i + 1][1], h)
					love.graphics.line(x1, y1, x2, y2)
				end

				love.graphics.setColor(theme.bg_nested)
				love.graphics.circle("fill", x0, y0, 3)
				love.graphics.setColor(0.9, 0.9, 0.9)
				love.graphics.circle("line", x0, y0, 3)

				love.graphics.setColor(theme.ui_text)

				local note_name = tuning.getName(note.pitch)
				util.drawText(note_name, x0 + 6, y0 - 9, w, 0)
			end
		end
	end
end

return Song
