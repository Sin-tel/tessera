local Transform = require("views/transform")
local View = require("view")
local engine = require("engine")
local tuning = require("tuning")

-- sub tools
local drag = require("tools/drag")
local edit = require("tools/edit")
local pan = require("tools/pan")
local scale = require("tools/scale")
local set_transport_time = require("tools/set_transport_time")

local RIBBON_H = 20

local Canvas = View.derive("Canvas")
Canvas.__index = Canvas

function Canvas.new()
	local self = setmetatable({}, Canvas)

	self.selected_tool = edit
	self.current_tool = self.selected_tool
	self.tool_active = false

	-- TODO: expose this as an option
	self.follow = false

	self.transform = Transform.new()

	return self
end

function Canvas:update()
	self.transform:update()

	if self:focus() then
		local mx, my = self:get_mouse()

		if 0 < my and my < RIBBON_H then
			-- hovering on ribbon
			mouse:set_cursor("hand")
		end

		if mouse.scroll then
			local zoom_factor = math.exp(0.15 * mouse.scroll)
			if not modifier_keys.ctrl then
				self.transform:zoom_x(mx, zoom_factor)
			end
			if not modifier_keys.shift then
				self.transform:zoom_y(my, zoom_factor)
			end
		end

		if self.current_tool.update then
			self.current_tool:update(self)
		end

		if self.tool_active then
			self.current_tool:mousedown(self)
		end
	end
end

function Canvas:draw()
	tessera.graphics.set_color(theme.bg_nested)
	tessera.graphics.rectangle("fill", 0, 0, self.w, self.h)

	-- draw grid
	local ix, iy = self.transform:inverse(0, 0)
	local ex, ey = self.transform:inverse(self.w, self.h)

	-- pitch grid
	local oct = tuning.generators[1]
	for i = math.floor((ey - 60) / oct), math.floor((iy - 60) / oct) do
		if self.transform.sy < -60 then
			tessera.graphics.set_color(theme.grid)
			for j, _ in ipairs(tuning.chromatic_table) do
				local py = self.transform:pitch(tuning.get_pitch(tuning.from_midi(j + 12 * i + 60)))
				tessera.graphics.line(0, py, self.w, py)
			end
		elseif self.transform.sy < -20 then
			tessera.graphics.set_color(theme.grid)
			for j, _ in ipairs(tuning.diatonic_table) do
				local py = self.transform:pitch(tuning.get_pitch(tuning.from_diatonic(j, i)))
				tessera.graphics.line(0, py, self.w, py)
			end
		elseif self.transform.sy < -2 then
			tessera.graphics.set_color(theme.grid_highlight)
			local py = self.transform:pitch(tuning.get_pitch({ i }))
			tessera.graphics.line(0, py, self.w, py)
		end
	end

	-- time grid
	local grid_t_res = 4 ^ math.floor(3.5 - math.log(self.transform.sx, 4))
	for i = math.floor(ix / grid_t_res) + 1, math.floor(ex / grid_t_res) do
		tessera.graphics.set_color(theme.grid)
		if i % 4 == 0 then
			tessera.graphics.set_color(theme.grid_highlight)
		end
		local px = self.transform:time(i * grid_t_res)
		tessera.graphics.line(px, 0, px, self.h)
	end

	-- if self.follow and px > self.w * 0.9 then
	if self.follow and engine.playing then
		self.transform.ox_ = -engine.time * self.transform.sx + self.w * 0.5
	end

	local w_scale = 0.3 * math.sqrt(-self.transform.sy * self.transform.sx)
	w_scale = math.min(20, w_scale)

	-- draw notes
	tessera.graphics.set_line_width(2.0)
	tessera.graphics.set_font_notes()
	tessera.graphics.set_font_size()

	for ch_index = #project.channels, 1, -1 do
		local ch = project.channels[ch_index]
		if ch.visible then
			local c_normal = tessera.graphics.get_color_hsv(ch.hue / 360, 0.75, 0.80)
			local c_select = tessera.graphics.get_color_hsv(ch.hue / 360, 0.50, 0.95)
			local c_lock = tessera.graphics.get_color_hsv(ch.hue / 360, 0.40, 0.40)
			local c_vert = tessera.graphics.get_color_hsv(ch.hue / 360, 0.70, 1.00)

			for _, note in ipairs(ch.notes) do
				local t_start = note.time
				local p_start = tuning.get_pitch(note.pitch)
				local x0 = self.transform:time(t_start)
				local y0 = self.transform:pitch(p_start)

				local x_end = self.transform:time(t_start + note.verts[#note.verts][1])

				-- cull only based on time
				if x_end > 0 and x0 < self.w then
					-- y0 offset by first vert
					local y0t = y0 + note.verts[1][2] * self.transform.sy

					-- velocity
					tessera.graphics.set_color_f(0.6, 0.6, 0.6)
					local vo = 32 * note.vel
					tessera.graphics.line(x0, y0t, x0, y0t - vo)
					tessera.graphics.line(x0 - 2, y0t - vo, x0 + 2, y0t - vo)

					-- note
					local c = c_normal
					if ch.lock then
						c = c_lock
					elseif selection.mask[note] then
						c = c_select
					end

					local lx = {}
					local ly = {}
					local lw = {}

					for i = 1, #note.verts do
						local x = self.transform:time(t_start + note.verts[i][1])
						local y = self.transform:pitch(p_start + note.verts[i][2])
						local w = note.verts[i][3] * w_scale

						table.insert(lx, x)
						table.insert(ly, y)
						table.insert(lw, w + 0.01)
					end

					-- draw temp lines for notes that are not yet finished
					if note.is_recording then
						local n = #note.verts
						local x = self.transform:time(engine.time)
						local y = self.transform:pitch(p_start + note.verts[n][2])
						local w = note.verts[n][3] * w_scale

						table.insert(lx, x)
						table.insert(ly, y)
						table.insert(lw, w)
					end

					-- tessera.graphics.set_color_f(c[1] * 0.5, c[2] * 0.5, c[3] * 0.5)
					tessera.graphics.set_color_f(c[1] * 0.5, c[2] * 0.5, c[3] * 0.5)
					tessera.graphics.polyline_w(lx, ly, lw)

					tessera.graphics.set_color(c)
					tessera.graphics.polyline(lx, ly)

					if self.transform.sy < -70 then
						tessera.graphics.set_color(c_vert)
						tessera.graphics.verts(lx, ly, 2)
					end

					-- note head
					tessera.graphics.set_color(c)
					tessera.graphics.circle("fill", x0, y0t, 3)

					-- note names
					if self.transform.sy < -20 then
						tessera.graphics.set_color(c)
						local note_name = tuning.get_name(note.pitch)
						tessera.graphics.text(note_name, x0 + 5, y0t - 20)
					end
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
						tessera.graphics.set_color_f(0.3, 0.3, 0.3)
						tessera.graphics.rectangle("fill", x1, y, x2 - x1, w)
					end
				end
			end
		end
	end
	tessera.graphics.set_line_width()

	-- top 'ribbon'
	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", 0, -1, self.w, RIBBON_H)
	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("line", 0, 0, self.w, RIBBON_H)

	-- playhead
	local px = self.transform:time(project.transport.start_time)
	tessera.graphics.set_color(theme.line_hover)
	tessera.graphics.line(px, 0, px, RIBBON_H)

	px = self.transform:time(engine.time)
	if project.transport.recording then
		tessera.graphics.set_color(theme.recording)
	else
		tessera.graphics.set_color(theme.widget)
	end
	tessera.graphics.line(px, 0, px, self.h)
	tessera.graphics.rectangle("fill", px - 2, 0, 4, RIBBON_H)

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
		-- select all
		local mask = {}
		for _, channel in ipairs(project.channels) do
			if channel.visible and not channel.lock then
				for _, note in ipairs(channel.notes) do
					mask[note] = true
				end
			end
		end
		selection.set(mask)
	elseif modifier_keys.ctrl and key == "x" then
		if not selection.is_empty() then
			local notes = selection.get_notes()
			clipboard.set(notes)
			local c = command.NoteDelete.new(notes)
			command.run_and_register(c)
			return true
		end
	elseif modifier_keys.ctrl and key == "c" then
		if not selection.is_empty() then
			local notes = selection.get_notes()
			clipboard.set(notes)
			return true
		end
	elseif modifier_keys.ctrl and key == "v" then
		if not clipboard.is_empty() then
			-- get notes and paste them
			local notes = util.clone(clipboard.notes)
			local c = command.NoteAdd.new(notes)
			-- drag tool is responsible for registering the actual command
			c:run()

			-- set selection to new notes
			selection.set_from_notes(notes)

			-- switch to drag mode
			self.current_tool = drag
			drag.mode = "paste"
			-- TODO: kind of hacky, should just have an init()
			self.current_tool:mousepressed(self)
			self.tool_active = true
			return true
		end
	elseif key == "delete" then
		if not selection.is_empty() then
			local notes = selection.get_notes()
			local c = command.NoteDelete.new(notes)
			command.run_and_register(c)
			return true
		end
	elseif key == "g" and not modifier_keys.any then
		if not selection.is_empty() then
			self.current_tool = drag
			self.current_tool:mousepressed(self)
			self.tool_active = true
			return true
		end
	elseif key == "s" and not modifier_keys.any then
		if not selection.is_empty() then
			self.current_tool = scale
			self.current_tool:mousepressed(self)
			self.tool_active = true
			return true
		end
	end

	if zoom_factor then
		self.transform:zoom_x(self.w * 0.25, zoom_factor)
		return true
	end

	if move_up then
		local prev_state = util.clone(selection.list)

		for _, v in ipairs(selection.list) do
			if modifier_keys.shift then
				tuning.move_octave(v.pitch, move_up)
			elseif modifier_keys.ctrl then
				tuning.move_chromatic(v.pitch, move_up)
			elseif modifier_keys.alt then
				tuning.move_comma(v.pitch, move_up)
			else
				tuning.move_diatonic(v.pitch, move_up)
			end
		end

		command.register(command.NoteUpdate.new(prev_state, selection.list))
		return true
	end

	if move_right then
		local prev_state = util.clone(selection.list)

		local move_amt = 1
		if modifier_keys.shift then
			move_amt = 0.25
		end
		if modifier_keys.alt then
			move_amt = 0.01
		end
		for _, v in ipairs(selection.list) do
			v.time = v.time + move_right * move_amt
		end

		command.register(command.NoteUpdate.new(prev_state, selection.list))
		return true
	end
end

function Canvas:dist_sq_note(note, mx, my)
	local d_max = math.huge

	local base_pitch = tuning.get_pitch(note.pitch)
	local t_start = note.time

	-- Calculate first point
	-- note.verts[1][1] is always 0
	local x1 = self.transform:time(t_start)
	local y1 = self.transform:pitch(base_pitch + note.verts[1][2])

	for k = 2, #note.verts do
		local vert = note.verts[k]

		local x2 = self.transform:time(t_start + vert[1])
		local y2 = self.transform:pitch(base_pitch + vert[2])

		-- Check segment distance
		local d_sq = util.segment_dist_sq(mx, my, x1, y1, x2, y2)
		if d_sq < d_max then
			d_max = d_sq
		end

		-- Shift for next segment
		x1, y1 = x2, y2
	end

	return d_max
end

function Canvas:find_closest_note(mx, my, max_distance)
	local closest
	local closest_ch

	-- use squared distance
	local dmax = max_distance ^ 2

	local c1 = 0
	local c2 = 0

	for ch_index, channel in ipairs(project.channels) do
		if channel.visible and not channel.lock then
			for _, note in ipairs(channel.notes) do
				-- Broad phase: bound by time range
				local t_start = note.time
				local t_end = note.time + note.verts[#note.verts][1]

				local x0 = self.transform:time(t_start)
				local x1 = self.transform:time(t_end)

				c1 = c1 + 1
				if mx >= x0 - max_distance and mx <= x1 + max_distance then
					-- Narrow phase: iterate over segments
					local note_dist = self:dist_sq_note(note, mx, my)
					c2 = c2 + 1

					-- We add a tiny bias based on distance to start of note (x0)
					-- to prefer the start of a note if two overlap exactly
					local tie_breaker = 0.0001 * math.abs(x0 - mx)

					if note_dist + tie_breaker < dmax then
						dmax = note_dist
						closest = note
						closest_ch = ch_index
					end
				end
			end
		end
	end

	return closest, closest_ch
end

function Canvas:find_closest_end(mx, my, max_distance)
	local closest
	local closest_ch
	local dmax = max_distance or math.huge
	for ch_index, channel in ipairs(project.channels) do
		if channel.visible and not channel.lock then
			for _, v in ipairs(channel.notes) do
				local t_end = v.time + v.verts[#v.verts][1]
				local p_end = tuning.get_pitch(v.pitch) + v.verts[#v.verts][2]
				local x0 = self.transform:time(t_end)
				local y0 = self.transform:pitch(p_end)

				local d = util.dist(x0, y0, mx, my)
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
	if self.tool_active then
		return
	end

	self.current_tool = self.selected_tool

	if mouse.button == 1 then
		local _, my = self:get_mouse()

		if 0 < my and my < RIBBON_H then
			-- clicked on top ribbon
			self.current_tool = set_transport_time
		end
	elseif mouse.button == 2 then
		return
	elseif mouse.button == 3 then
		self.current_tool = pan
	end

	self.current_tool:mousepressed(self)
	self.tool_active = true
end

function Canvas:mousereleased()
	if mouse.button == 2 then
		return
	end
	self.current_tool:mousereleased(self)
	self.tool_active = false
	self.current_tool = self.selected_tool
end

return Canvas
