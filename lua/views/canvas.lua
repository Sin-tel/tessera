local Transform = require("views/transform")
local View = require("view")
local engine = require("engine")
local hsluv = require("lib/hsluv")
local tuning = require("tuning")

-- sub tools
local drag = require("tools/drag")
local edit = require("tools/edit")
local pan = require("tools/pan")
local scale = require("tools/scale")
local set_transport_time = require("tools/set_transport_time")

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
		end
		tessera.graphics.set_color(theme.grid_highlight)
		local py = self.transform:pitch(tuning.get_pitch({ i }))
		tessera.graphics.line(0, py, self.w, py)
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
		self.transform.ox_ = -project.transport.time * self.transform.sx + self.w * 0.5
	end

	local w_scale = math.min(12, -self.transform.sy)

	-- draw notes
	tessera.graphics.set_font(resources.fonts.notes)
	for ch_index, ch in ipairs(project.channels) do
		if ch.visible then
			local c_normal = hsluv.hsluv_to_rgb({ ch.hue, 80.0, 60.0 })
			local c_select = hsluv.hsluv_to_rgb({ ch.hue, 50.0, 80.0 })
			local c_lock = hsluv.hsluv_to_rgb({ ch.hue, 40.0, 40.0 })

			for _, note in ipairs(ch.notes) do
				local t_start = note.time
				local p_start = tuning.get_pitch(note.pitch)
				local x0 = self.transform:time(t_start)
				local y0 = self.transform:pitch(p_start)

				-- velocity
				tessera.graphics.set_color(0.6, 0.6, 0.6)
				local vo = 32 * note.vel
				tessera.graphics.line(x0, y0, x0, y0 - vo)
				tessera.graphics.line(x0 - 2, y0 - vo, x0 + 2, y0 - vo)

				-- note
				local c = c_normal
				if ch.lock then
					c = c_lock
				end
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
					if w1 > 1.0 or w2 > 1.0 then
						tessera.graphics.set_color(0.3, 0.3, 0.3)
						tessera.graphics.polygon(
							"fill",
							x1,
							y1 + w1,
							x2,
							y2 + w2,
							x2,
							y2 - w2,
							x1,
							y1 - w1,
							x1,
							y1 + w1
						)
					end
					tessera.graphics.set_color(c)
					tessera.graphics.line(x1, y1, x2, y2)
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
					if w1 > 1.0 or w2 > 1.0 then
						tessera.graphics.set_color(0.3, 0.3, 0.3)
						tessera.graphics.polygon(
							"fill",
							x1,
							y1 + w1,
							x2,
							y2 + w2,
							x2,
							y2 - w2,
							x1,
							y1 - w1,
							x1,
							y1 + w1
						)
					end
					tessera.graphics.set_color(c)
					tessera.graphics.line(x1, y1, x2, y2)
				end

				-- note head
				tessera.graphics.set_color(theme.bg_nested)
				tessera.graphics.circle("fill", x0, y0, 3)
				tessera.graphics.set_color(c)
				tessera.graphics.circle("line", x0, y0, 3)

				-- note names
				if self.transform.sy < -20 then
					tessera.graphics.set_color(c)
					local note_name = tuning.get_name(note.pitch)
					util.draw_text(note_name, x0 + 5, y0 - 10, self.w, 0)
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
						tessera.graphics.set_color(0.3, 0.3, 0.3)
						tessera.graphics.rectangle("fill", x1, y, x2 - x1, w)
					end
				end
			end
		end
	end
	tessera.graphics.set_line_width(1)

	-- top 'ribbon'
	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("fill", 0, -1, self.w, 16)
	tessera.graphics.set_color(theme.background)
	tessera.graphics.rectangle("line", 0, 0, self.w, 16)

	-- playhead
	local px = self.transform:time(project.transport.time)
	if project.transport.recording then
		tessera.graphics.set_color(theme.recording)
	else
		tessera.graphics.set_color(theme.widget)
	end
	tessera.graphics.line(px, 0, px, self.h)

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
			command.run_and_register(c)

			-- set selection to new notes
			selection.set_from_notes(notes)

			-- switch to drag mode
			self.current_tool = drag
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

		for k, v in ipairs(selection.list) do
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
		for k, v in ipairs(selection.list) do
			v.time = v.time + move_right * move_amt
		end

		command.register(command.NoteUpdate.new(prev_state, selection.list))
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
				local p_start = tuning.get_pitch(v.pitch)
				local x0 = self.transform:time(t_start)
				local x1 = self.transform:time(t_end)
				local y0 = self.transform:pitch(p_start)

				-- Assuming note is a horizontal line, project target onto it
				local x_proj = util.clamp(mx, x0, x1)

				-- Use projected distance, with distance to start as a tie-breaker
				local d = util.dist(x_proj, y0, mx, my) + 0.001 * math.abs(x0 - mx)
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

function Canvas:find_closest_end(mx, my, max_distance)
	local closest
	local closest_ch
	local dmax = max_distance or math.huge
	for ch_index, channel in ipairs(project.channels) do
		if channel.visible and not channel.lock then
			for i, v in ipairs(channel.notes) do
				local t_end = v.time + v.verts[#v.verts][1]
				local p_start = tuning.get_pitch(v.pitch)
				local x0 = self.transform:time(t_end)
				local y0 = self.transform:pitch(p_start)

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

		if my > 0 and my < 16 then
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
