local tuning = require("tuning")
local engine = require("engine")
local View = require("view")
local Transform = require("views/transform")

local select_rect = require("tools/select_rect")
local pan = require("tools/pan")
local drag = require("tools/drag")

local hsluv = require("lib/hsluv")

local Canvas = View.derive("Canvas")
Canvas.__index = Canvas

function Canvas.new()
	local self = setmetatable({}, Canvas)

	self.current_tool = pan
	self.selected_tool = select_rect

	-- TODO: expose this as an option
	self.follow = false

	self.transform = Transform.new()

	return self
end

function Canvas:update()
	self.transform:update()

	if self:focus() then
		local mx, my = self:getMouse()

		if mouse.scroll then
			local zoom_factor = math.exp(0.15 * mouse.scroll)
			if not modifier_keys.ctrl then
				self.transform:zoom_x(mx, zoom_factor)
			end
			if not modifier_keys.shift then
				self.transform:zoom_y(my, zoom_factor)
			end
		end

		if mouse.button == 1 and my > 0 and my < 16 then
			-- move playhead when clicking on ribbon
			-- TODO: action should be triggered when mouse pressed in ribbon
			local new_time = self.transform:time_inv(mx)

			project.transport.start_time = new_time
			engine.seek(new_time)
		elseif mouse.button then
			self.current_tool:mousedown(self)
		end
	end
end

function Canvas:draw()
	love.graphics.setColor(theme.bg_nested)
	love.graphics.rectangle("fill", 0, 0, self.w, self.h)

	-- draw grid
	local ix, iy = self.transform:inverse(0, 0)
	local ex, ey = self.transform:inverse(self.w, self.h)

	-- pitch grid
	local oct = tuning.generators[1]
	for i = math.floor((ey - 60) / oct), math.floor((iy - 60) / oct) do
		if self.transform.sy < -60 then
			love.graphics.setColor(theme.grid)
			for j, _ in ipairs(tuning.chromatic_table) do
				local py = self.transform:pitch(tuning.getPitch(tuning.fromMidi(j + 12 * i + 60)))
				love.graphics.line(0, py, self.w, py)
			end
		elseif self.transform.sy < -20 then
			love.graphics.setColor(theme.grid)
			for j, _ in ipairs(tuning.diatonic_table) do
				local py = self.transform:pitch(tuning.getPitch(tuning.fromDiatonic(j, i)))
				love.graphics.line(0, py, self.w, py)
			end
		end
		love.graphics.setColor(theme.grid_highlight)
		local py = self.transform:pitch(tuning.getPitch({ i }))
		love.graphics.line(0, py, self.w, py)
	end

	-- time grid
	local grid_t_res = 4 ^ math.floor(3.5 - math.log(self.transform.sx, 4))
	for i = math.floor(ix / grid_t_res) + 1, math.floor(ex / grid_t_res) do
		love.graphics.setColor(theme.grid)
		if i % 4 == 0 then
			love.graphics.setColor(theme.grid_highlight)
		end
		local px = self.transform:time(i * grid_t_res)
		love.graphics.line(px, 0, px, self.h)
	end

	-- if self.follow and px > self.w * 0.9 then
	if self.follow and engine.playing then
		self.transform.ox_ = -project.transport.time * self.transform.sx + self.w * 0.5
	end

	local w_scale = math.min(12, -self.transform.sy)

	-- draw notes
	love.graphics.setFont(resources.fonts.notes)
	for ch_index, ch in ipairs(project.channels) do
		if ch.visible then
			local c_normal = hsluv.hsluv_to_rgb({ ch.hue, 80.0, 60.0 })
			local c_select = hsluv.hsluv_to_rgb({ ch.hue, 50.0, 80.0 })

			for _, note in ipairs(ch.notes) do
				local t_start = note.time
				local p_start = tuning.getPitch(note.pitch)
				local x0 = self.transform:time(t_start)
				local y0 = self.transform:pitch(p_start)

				-- velocity
				love.graphics.setColor(0.6, 0.6, 0.6)
				local vo = 32 * note.vel
				love.graphics.line(x0, y0, x0, y0 - vo)
				love.graphics.line(x0 - 2, y0 - vo, x0 + 2, y0 - vo)

				-- note
				local c = c_normal
				if selection.mask[note] then
					c = c_select
				end
				for i = 1, #note.verts - 1 do
					local x1 = self.transform:time(t_start + note.verts[i][1])
					local x2 = self.transform:time(t_start + note.verts[i + 1][1])
					local y1 = self.transform:pitch(p_start + note.verts[i][2])
					local y2 = self.transform:pitch(p_start + note.verts[i + 1][2])
					local w1 = note.verts[i][3] * w_scale
					local w2 = note.verts[i + 1][3] * w_scale
					love.graphics.setColor(0.3, 0.3, 0.3)

					love.graphics.polygon("fill", x1, y1 + w1, x2, y2 + w2, x2, y2 - w2, x1, y1 - w1, x1, y1 + w1)
					love.graphics.setColor(c)
					love.graphics.line(x1, y1, x2, y2)
				end

				-- draw temp lines for notes that are not yet finished
				if note.is_recording then
					local n = #note.verts
					local x1 = self.transform:time(t_start + note.verts[n][1])
					local x2 = self.transform:time(project.transport.time)
					local y1 = self.transform:pitch(p_start + note.verts[n][2])
					local y2 = y1
					local w1 = note.verts[n][3] * w_scale
					local w2 = w1
					love.graphics.setColor(0.3, 0.3, 0.3)
					love.graphics.polygon("fill", x1, y1 + w1, x2, y2 + w2, x2, y2 - w2, x1, y1 - w1, x1, y1 + w1)
					love.graphics.setColor(c)
					love.graphics.line(x1, y1, x2, y2)
				end

				-- note head
				love.graphics.setColor(theme.bg_nested)
				love.graphics.circle("fill", x0, y0, 3)
				love.graphics.setColor(c)
				love.graphics.circle("line", x0, y0, 3)

				-- note names
				if self.transform.sy < -20 then
					love.graphics.setColor(c)
					local note_name = tuning.getName(note.pitch)
					util.drawText(note_name, x0 + 5, y0 - 10, self.w, 0)
				end
			end

			-- sustain pedal
			if ch.control.sustain then
				local w = w_scale
				local y = self.h - w
				for i = 1, #ch.control.sustain - 1 do
					local c = ch.control.sustain[i]
					local c2 = ch.control.sustain[i + 1]

					if c.value and not c2.value then
						local x1 = self.transform:time(c.time)
						local x2 = self.transform:time(c2.time)
						love.graphics.setColor(0.3, 0.3, 0.3)
						love.graphics.rectangle("fill", x1, y, x2 - x1, w)
					end
				end
			end
		end
	end
	-- top 'ribbon'
	love.graphics.setColor(theme.background)
	love.graphics.rectangle("fill", 0, -1, self.w, 16)
	love.graphics.setColor(theme.background)
	love.graphics.rectangle("line", 0, 0, self.w, 16)

	-- playhead
	local px = self.transform:time(project.transport.time)
	if project.transport.recording then
		love.graphics.setColor(theme.recording)
	else
		love.graphics.setColor(theme.widget)
	end
	love.graphics.line(px, 0, px, self.h)

	self.current_tool:draw(self)
end

function Canvas:keypressed(key)
	local zoom_factor
	local move_up
	local move_right
	if key == "kp+" then
		zoom_factor = math.sqrt(2)
	elseif key == "kp-" then
		zoom_factor = 1 / math.sqrt(2)
	elseif key == "up" then
		move_up = 1
	elseif key == "down" then
		move_up = -1
	elseif key == "right" then
		move_right = 1
	elseif key == "left" then
		move_right = -1
	elseif key == "a" and modifier_keys.ctrl then
		-- select all in channel
		local mask = {}
		if selection.ch_index then
			local channel = project.channels[selection.ch_index]
			for i, v in ipairs(channel.notes) do
				mask[v] = true
			end
		end
		selection.setNormal(mask)
	elseif key == "delete" then
		-- remove selected notes

		for _, channel in ipairs(project.channels) do
			if channel.visible and not channel.lock then
				for i = #channel.notes, 1, -1 do
					if selection.mask[channel.notes[i]] then
						table.remove(channel.notes, i)
					end
				end
			end
		end
		return true
	end

	if zoom_factor then
		self.transform:zoom_x(self.w * 0.25, zoom_factor)
		return true
	end

	if move_up then
		for k, v in ipairs(selection.list) do
			if modifier_keys.shift then
				tuning.moveOctave(v.pitch, move_up)
			elseif modifier_keys.ctrl then
				tuning.moveChromatic(v.pitch, move_up)
			elseif modifier_keys.alt then
				tuning.moveComma(v.pitch, move_up)
			else
				tuning.moveDiatonic(v.pitch, move_up)
			end
		end
		return true
	end

	if move_right then
		local move_amt = 1
		if modifier_keys.shift then
			move_amt = 0.25
		end
		if modifier_keys.alt then
			move_amt = 0.01
		end
		for k, v in ipairs(selection.list) do
			v.time = v.time + move_right * move_amt
		end
		return true
	end
end

function Canvas:find_closest_note(mx, my, max_distance)
	local closest
	local closest_ch
	local dmax = max_distance or math.huge
	for ch_index, channel in ipairs(project.channels) do
		if channel.visible and not channel.lock then
			for i, v in ipairs(channel.notes) do
				local t_start = v.time
				local t_end = v.time + v.verts[#v.verts][1]
				local p_start = tuning.getPitch(v.pitch)
				local x0 = self.transform:time(t_start)
				local x1 = self.transform:time(t_end)
				local y0 = self.transform:pitch(p_start)

				-- Assuming note is a horizontal line, project target onto it
				local x_proj = util.clamp(mx, x0, x1)

				-- Use projected distance, with distance to start as a tie-breaker
				local d = util.dist(x_proj, y0, mx, my) + 0.1 * math.abs(x0 - mx)
				if d < dmax then
					dmax = d
					closest = v
					closest_ch = ch_index
				end
			end
		end
	end

	return closest, closest_ch
end

function Canvas:mousepressed()
	self.current_tool = self.selected_tool

	if mouse.button == 1 then
		-- Check if click on note
		local mx, my = self:getMouse()

		local closest, closest_ch = self:find_closest_note(mx, my, 24)

		-- If we click on a note, switch to drag mode instead of select
		if closest then
			drag:set_note_origin(closest)
			self.current_tool = drag

			-- If not part of selection already, change selection to just the note we clicked
			if not selection.mask[closest] then
				selection.setNormal({ [closest] = true })
				selection.ch_index = closest_ch
			end
		end
	elseif mouse.button == 3 then
		self.current_tool = pan
	end

	self.current_tool:mousepressed(self)
end

function Canvas:mousereleased()
	self.current_tool:mousereleased(self)
end

return Canvas
