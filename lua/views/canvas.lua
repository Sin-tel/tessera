local Menu = require("views.canvas_menu")
local Transform = require("views/transform")
local Ui = require("ui.ui")
local View = require("view")
local engine = require("engine")
local tuning = require("tuning")
require("table.new")

-- sub tools
local drag = require("tools/drag")
local edit = require("tools/edit")
local pan = require("tools/pan")
local pen = require("tools/pen")
local scale = require("tools/scale")
local set_transport_time = require("tools/set_transport_time")

local Button = {}
Button.__index = Button

local BUTTON_S = Ui.scale(40)
local RIBBON_H = Ui.scale(20)

local Canvas = View.derive("Canvas")
Canvas.__index = Canvas

function Canvas.new()
	local self = setmetatable({}, Canvas)

	self.selected_tool = edit
	self.current_tool = self.selected_tool
	self.tool_active = false

	self.transform = Transform.new()

	self.tool_buttons = {}
	local x1, y1 = 16, 16 + RIBBON_H
	table.insert(self.tool_buttons, Button.new(x1, y1, tessera.icon.edit, edit))
	y1 = y1 + BUTTON_S + Ui.PAD
	table.insert(self.tool_buttons, Button.new(x1, y1, tessera.icon.pen, pen))

	return self
end

function Canvas:update()
	self.transform:update()

	-- follow playhead
	if project.settings.follow and engine.playing then
		local px = self.transform.ox_ + engine.time * self.transform.sx

		-- if px < self.w * 0.1 then
		-- 	self.transform.ox_ = -engine.time * self.transform.sx + self.w * 0.1
		-- elseif px > self.w * 0.9 then
		-- 	self.transform.ox_ = -engine.time * self.transform.sx + self.w * 0.9
		-- else
		-- 	self.transform.ox_ = self.transform.ox_ - engine.frame_time * self.transform.sx
		-- end

		if px < self.w * 0.15 or px > self.w * 0.85 then
			self.transform.ox_ = -engine.time * self.transform.sx + self.w * 0.15
		end
	end
	if self:focus() then
		local mx, my = self:get_mouse()

		if my > 0 then
			if mouse.scroll then
				local zoom_factor = math.exp(0.15 * mouse.scroll)
				if not modifier_keys.ctrl then
					self.transform:zoom_x(mx, zoom_factor)
				end
				if not modifier_keys.shift then
					self.transform:zoom_y(my, zoom_factor)
				end
			end
			if mouse.scroll_x then
				self.transform.ox_ = self.transform.ox_ + mouse.scroll_x * self.w * 0.2
			end
			if self.tool_active then
				if self.current_tool.update then
					self.current_tool:update(self)
				end
				self.current_tool:mousedown(self)
				return
			end
			if my < RIBBON_H then
				-- hovering on ribbon
				mouse:set_cursor("hand")
				return
			end
			for _, v in ipairs(self.tool_buttons) do
				if v:update(self, mx, my) then
					return
				end
			end
			if self.current_tool.update then
				self.current_tool:update(self)
				return
			end
		end
	end
end

function Canvas:draw_channel(ch, w_scale)
	if not ch.visible then
		return
	end
	local v_scale = -self.transform.sy
	v_scale = math.min(32, v_scale)

	local note_head_size = Ui.scale(3.5)
	local v_size = Ui.scale(2.5)
	local label_x = Ui.scale(5)
	local label_y = Ui.scale(20)

	local c_normal = tessera.graphics.get_color_hsv(ch.hue / 360, 0.75, 0.80)
	local c_select = tessera.graphics.get_color_hsv(ch.hue / 360, 0.50, 0.95)
	local c_lock = tessera.graphics.get_color_hsv(ch.hue / 360, 0.40, 0.40)
	local c_vert = tessera.graphics.get_color_hsv(ch.hue / 360, 0.70, 1.00)

	for _, note in ipairs(ch.notes) do
		local t_start = note.time
		local p_start = tuning.get_pitch(note.interval)
		local x0 = self.transform:time(t_start)
		local y0 = self.transform:pitch(p_start)

		local x_end = self.transform:time(t_start + note.verts[#note.verts][1])

		-- cull only based on time
		if x_end > 0 and x0 < self.w then
			-- y0 offset by first vert
			local y0t = y0 + note.verts[1][2] * self.transform.sy

			-- velocity
			if v_scale > 10 then
				tessera.graphics.set_color_f(0.6, 0.6, 0.6)
				local vo = note.vel * v_scale
				tessera.graphics.line(x0, y0t, x0, y0t - vo)
				tessera.graphics.line(x0 - v_size, y0t - vo, x0 + v_size, y0t - vo)
			end

			-- note
			local c = c_normal
			if ch.lock then
				c = c_lock
			elseif selection.mask[note] then
				c = c_select
			end

			local n = #note.verts
			local lx = table.new(n, 0)
			local ly = table.new(n, 0)
			local lw = table.new(n, 0)

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
				local x = self.transform:time(engine.time)
				local y = self.transform:pitch(p_start + note.verts[n][2])
				local w = note.verts[n][3] * w_scale

				table.insert(lx, x)
				table.insert(ly, y)
				table.insert(lw, w + 0.01)
			end

			if w_scale > 5 then
				tessera.graphics.set_color_f(c[1] * 0.5, c[2] * 0.5, c[3] * 0.5)
				tessera.graphics.polyline_w(lx, ly, lw)
			end

			tessera.graphics.set_color(c)
			tessera.graphics.polyline(lx, ly)

			if self.transform.sy < -70 then
				tessera.graphics.set_color(c_vert)
				tessera.graphics.verts(lx, ly, 2)
			end

			-- note head
			tessera.graphics.set_color(c)
			tessera.graphics.circle("fill", x0, y0t, note_head_size)

			-- note names
			if self.transform.sy < -20 then
				tessera.graphics.set_color(c)
				local note_name = tuning.get_name(note.interval)
				tessera.graphics.text(note_name, x0 + label_x, y0t - label_y)
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

function Canvas:draw_pitch_grid(t)
	local iy = self.transform:pitch_inv(0)
	local ey = self.transform:pitch_inv(self.h)

	local oct = tuning.generators[1]
	for i = math.floor((ey - 60) / oct), math.floor((iy - 60) / oct) do
		if t == "octave" then
			local py = self.transform:pitch(tuning.get_pitch({ i }))
			tessera.graphics.line(0, py, self.w, py)
		else
			for j, _ in ipairs(t) do
				local py = self.transform:pitch(tuning.get_pitch(tuning.from_table(t, j + #t * i)))
				tessera.graphics.line(0, py, self.w, py)
			end
		end
	end
end

function Canvas:draw()
	tessera.graphics.set_color(theme.bg_nested)
	tessera.graphics.rectangle("fill", 0, 0, self.w, self.h)

	tessera.graphics.set_line_width(1)
	-- pitch grid
	if self.transform.sy < -124 then
		-- fine
		tessera.graphics.set_color(theme.grid)
		self:draw_pitch_grid(tuning.fine_table)

		tessera.graphics.set_color(theme.grid_highlight)
		self:draw_pitch_grid(tuning.chromatic_table)
	elseif self.transform.sy < -48 then
		-- medium (chromatic)
		tessera.graphics.set_color(theme.grid)
		self:draw_pitch_grid(tuning.chromatic_table)

		tessera.graphics.set_color(theme.grid_highlight)
		self:draw_pitch_grid(tuning.diatonic_table)
	elseif self.transform.sy < -28 then
		-- coarse (diatonic)
		tessera.graphics.set_color(theme.grid)
		self:draw_pitch_grid(tuning.diatonic_table)

		tessera.graphics.set_color(theme.grid_highlight)
		self:draw_pitch_grid("octave")
	end

	tessera.graphics.set_line_width(1.5)
	if self.transform.sy < -8 then
		-- octaves only
		tessera.graphics.set_color(theme.grid_highlight)
		self:draw_pitch_grid("octave")
	end

	-- time grid
	tessera.graphics.set_line_width(1.0)
	local ix = self.transform:time_inv(0)
	local ex = self.transform:time_inv(self.w)
	local grid_t_res = 4 ^ math.floor(3.8 - math.log(self.transform.sx, 4))
	for i = math.floor(ix / grid_t_res) + 1, math.floor(ex / grid_t_res) do
		tessera.graphics.set_color(theme.grid)
		if i % 4 == 0 then
			tessera.graphics.set_color(theme.grid_highlight)
		end
		local px = self.transform:time(i * grid_t_res)
		tessera.graphics.line(px, 0, px, self.h)
	end

	-- draw notes
	local w_scale = 0.3 * math.sqrt(-self.transform.sy * self.transform.sx)
	w_scale = math.min(20, w_scale)

	tessera.graphics.set_line_width(2.5)
	tessera.graphics.set_font_notes()
	tessera.graphics.set_font_size()

	for ch_index = #project.channels, 1, -1 do
		if ch_index ~= selection.ch_index then
			local ch = project.channels[ch_index]
			self:draw_channel(ch, w_scale)
		end
	end
	-- make sure currently active channel is on top
	if selection.ch_index then
		local ch = project.channels[selection.ch_index]
		self:draw_channel(ch, w_scale)
	end

	-- top 'ribbon'
	tessera.graphics.set_line_width()
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

	for _, v in ipairs(self.tool_buttons) do
		v:draw(self)
	end
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
	elseif key == "tab" then
		-- switch between edit and draw mode
		if self.selected_tool == edit then
			self:select_tool(pen)
		else
			self:select_tool(edit)
		end
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

			-- check if all notes belong to a single channel
			-- TODO: this behaviour might be a bit confusing, but it's useful
			local single_channel = false
			for ch_index in pairs(notes) do
				if #notes[ch_index] > 0 then
					if single_channel then
						single_channel = false
						break
					else
						single_channel = ch_index
					end
				end
			end
			-- move to selected channel
			if single_channel and selection.ch_index then
				local new_notes = {}
				new_notes[selection.ch_index] = util.clone(notes[single_channel])
				notes = new_notes
			end

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
	elseif key == "delete" or key == "backspace" then
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
	elseif key == "kp." or key == "." then
		self:auto_zoom()
	end

	if zoom_factor then
		self.transform:zoom_x(self.w * 0.25, zoom_factor)
		return true
	end

	if move_up then
		local prev_state = util.clone(selection.list)

		local delta = tuning.new_interval()

		if modifier_keys.shift then
			delta = tuning.mul(tuning.octave, move_up)
		elseif modifier_keys.ctrl then
			delta = tuning.mul(tuning.chroma, move_up)
		elseif modifier_keys.alt then
			delta = tuning.mul(tuning.comma, move_up)
		else
			-- we use the lowest note as the base.
			-- TODO: once there's a more sophisticated key system, query that
			local d_min = math.huge
			local base = nil
			for _, note in ipairs(selection.list) do
				local n = tuning.get_pitch(note.interval)
				if n < d_min then
					d_min = n
					base = note
				end
			end

			if base then
				local diatonic = tuning.diatonic_table
				local n = tuning.get_index(#diatonic, base.interval)
				local p_origin = tuning.from_table(diatonic, n)
				delta = tuning.from_table(diatonic, n + move_up)
				delta = tuning.sub(delta, p_origin)
			end
		end
		for _, v in ipairs(selection.list) do
			v.interval = tuning.add(v.interval, delta)
		end

		command.register(command.NoteUpdate.new(prev_state, selection.list))
		return true
	end

	if move_right then
		local prev_state = util.clone(selection.list)

		local move_amt = 1
		if modifier_keys.shift then
			move_amt = 1 / 4
		elseif modifier_keys.ctrl then
			move_amt = 1 / 16
		elseif modifier_keys.alt then
			move_amt = 1 / 128
		end
		for _, v in ipairs(selection.list) do
			v.time = v.time + move_right * move_amt
		end

		command.register(command.NoteUpdate.new(prev_state, selection.list))
		return true
	end
end

function Canvas:keyreleased(key)
	--
end

function Canvas:auto_zoom()
	local all_notes = selection.is_empty()

	local t_min = math.huge
	local t_max = -math.huge
	local p_min = math.huge
	local p_max = -math.huge
	for _, channel in ipairs(project.channels) do
		if channel.visible then
			for _, note in ipairs(channel.notes) do
				if all_notes or selection.mask[note] then
					local t_start = note.time
					local t_end = note.time + note.verts[#note.verts][1]

					local p = tuning.get_pitch(note.interval)

					if t_start < t_min then
						t_min = t_start
					end
					if t_end > t_max then
						t_max = t_end
					end
					if p < p_min then
						p_min = p
					end
					if p > p_max then
						p_max = p
					end
				end
			end
		end
	end
	if t_min < t_max then
		self.transform.sx_ = 0.7 * self.w / (t_max - t_min)
		self.transform.ox_ = -t_min * self.transform.sx_ + 0.15 * self.w
	end

	if p_min < p_max then
		self.transform.sy_ = -0.7 * self.h / (p_max - p_min)
		self.transform.oy_ = -p_max * self.transform.sy_ + 0.15 * self.h
	end

	-- if there's only a single note, center
	if p_max == p_min then
		self.transform.sy_ = -48
		self.transform.oy_ = -p_max * self.transform.sy_ + 0.5 * self.h
	end
end

function Canvas:dist_sq_note(note, mx, my)
	local d_max = math.huge

	local base_pitch = tuning.get_pitch(note.interval)
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
					note_dist = note_dist + 0.0001 * math.abs(x0 - mx)

					if note_dist < dmax then
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
			for _, note in ipairs(channel.notes) do
				local t_end = note.time + note.verts[#note.verts][1]
				local p_end = tuning.get_pitch(note.interval) + note.verts[#note.verts][2]
				local x0 = self.transform:time(t_end)
				local y0 = self.transform:pitch(p_end)

				local d = util.dist(x0, y0, mx, my)
				if d < dmax then
					dmax = d
					closest = note
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

	for _, v in ipairs(self.tool_buttons) do
		if v:mousepressed(self) then
			return
		end
	end

	local _, my = self:get_mouse()
	if my > 0 then
		self.current_tool = self.selected_tool

		if mouse.button == 3 or (mouse.button == 1 and modifier_keys.alt and modifier_keys.ctrl) then
			self.current_tool = pan
		elseif mouse.button == 1 then
			if my < RIBBON_H then
				-- clicked on top ribbon
				self.current_tool = set_transport_time
			end
		elseif mouse.button == 2 then
			workspace:set_overlay(Menu.new(self))
			return
		end

		self.current_tool:mousepressed(self)
		self.tool_active = true
	end
end

function Canvas:mousereleased()
	if mouse.button == 2 then
		return
	end
	if not self.tool_active then
		return
	end

	self.current_tool:mousereleased(self)
	self.tool_active = false
	self.current_tool = self.selected_tool
end

function Canvas:select_tool(tool)
	self.selected_tool = tool
	self.current_tool = tool
end

--- Tool buttons

function Button.new(x, y, icon, tool)
	local self = setmetatable({}, Button)
	self.x = x
	self.y = y
	self.w = BUTTON_S
	self.h = BUTTON_S
	self.icon = icon
	self.tool = tool
	return self
end

function Button:update(canvas)
	local mx, my = canvas:get_mouse()
	if util.hit(self, mx, my) then
		mouse:set_cursor("hand")
		return true
	end
end

function Button:mousepressed(canvas)
	local mx, my = canvas:get_mouse()

	if util.hit(self, mx, my) then
		canvas:select_tool(self.tool)
		return true
	end
end

function Button:draw(canvas)
	if canvas.selected_tool == self.tool then
		tessera.graphics.set_color(theme.bg_highlight)
	else
		tessera.graphics.set_color(theme.bg_menu)
	end

	tessera.graphics.rectangle("fill", self.x, self.y, self.w, self.h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(theme.line)
	tessera.graphics.rectangle("line", self.x, self.y, self.w, self.h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(theme.white)
	tessera.graphics.draw_path(self.icon, self.x, self.y)
end

return Canvas
