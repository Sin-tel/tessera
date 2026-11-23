-- Erases verts under the mouse in the active channel, splitting notes where it cuts through.
-- Pieces under two verts are removed.

local Ui = require("ui.ui")
local curve = require("curve")
local tuning = require("tuning")

local erase = {}

erase.name = "erase"
erase.radius = Ui.scale(12)

function erase:mousepressed(canvas)
	self.notes = nil
	local channel, ch_index = selection.active_channel()
	if not channel then
		return
	end

	self.ch_index = ch_index
	self.notes = channel.notes
	-- touched originals -> snapshot
	self.prev = {}
	-- new pieces still in the channel
	self.added = {}
	self.px, self.py = canvas:get_mouse()
	self:mousedown(canvas)
end

local function piece_of(note, verts, i1, i2)
	local t0 = verts[i1][1]
	local out = {}
	for i = i1, i2 do
		local v = verts[i]
		table.insert(out, { v[1] - t0, v[2], v[3] })
	end
	return { interval = note.interval, time = note.time + t0, vel = note.vel, verts = out }
end

-- Runs of `verts` outside the stroke segment, as pieces. Nil if nothing is erased.
function erase:cut(canvas, note, verts, mx, my)
	local transform = canvas.transform
	local base_pitch = tuning.get_pitch(note.interval)
	local r_sq = self.radius ^ 2

	local pieces = {}
	local start
	local erased = false
	for i = 1, #verts + 1 do
		local inside = false
		if i <= #verts then
			local x = transform:time(note.time + verts[i][1])
			local y = transform:pitch(base_pitch + verts[i][2])
			inside = util.segment_dist_sq(x, y, self.px, self.py, mx, my) < r_sq
		end
		if inside or i > #verts then
			erased = erased or inside
			if start and i - 1 > start then
				table.insert(pieces, piece_of(note, verts, start, i - 1))
			end
			start = nil
		elseif not start then
			start = i
		end
	end
	return erased and pieces
end

function erase:mousedown(canvas)
	if not self.notes then
		return
	end
	local mx, my = canvas:get_mouse()
	local reach = self.radius + util.dist(self.px, self.py, mx, my)

	for i = #self.notes, 1, -1 do
		local note = self.notes[i]
		if canvas:dist_sq_note(note, mx, my) < reach * reach then
			-- notes are only resampled once something is erased from them
			local fresh = not (self.prev[note] or self.added[note])
			local verts = fresh and curve.subdivide(note.verts) or note.verts
			local pieces = self:cut(canvas, note, verts, mx, my)
			if pieces then
				if fresh then
					self.prev[note] = util.clone(note)
				end
				-- the first piece stays in the note
				local first = table.remove(pieces, 1)
				if first then
					note.time, note.verts = first.time, first.verts
				else
					table.remove(self.notes, i)
					self.added[note] = nil
				end
				for _, piece in ipairs(pieces) do
					table.insert(self.notes, piece)
					self.added[piece] = true
				end
			end
		end
	end
	self.px, self.py = mx, my
end

function erase:mousereleased()
	if not self.notes then
		return
	end
	local in_channel = {}
	for _, note in ipairs(self.notes) do
		in_channel[note] = true
	end

	local prev_state, updated, deleted, added = {}, {}, {}, {}
	for note, prev in pairs(self.prev) do
		if in_channel[note] then
			note.verts = curve.simplify(note.verts)
			table.insert(prev_state, prev)
			table.insert(updated, note)
		else
			-- restored, so undo puts back the original
			for key, value in pairs(prev) do
				note[key] = value
			end
			table.insert(deleted, note)
		end
	end
	for note in pairs(self.added) do
		note.verts = curve.simplify(note.verts)
		table.insert(added, note)
	end
	self.notes = nil

	-- already applied, only registered
	local ch = self.ch_index
	command.register(command.Group.new({
		command.NoteUpdate.new(prev_state, updated),
		command.NoteDelete.new({ [ch] = deleted }),
		command.NoteAdd.new({ [ch] = added }),
	}))
	selection.remove_inactive()
end

function erase:draw(canvas)
	local mx, my = canvas:get_mouse()
	tessera.graphics.set_color(theme.text_tip)
	tessera.graphics.circle("line", mx, my, self.radius)
end

return erase
