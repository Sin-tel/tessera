local adjust_velocity = require("tools/adjust_velocity")
local drag = require("tools/drag")
local drag_end = require("tools/drag_end")
local select_rect = require("tools/select_rect")

local DRAG_END_DIST = 12
local NOTE_EDIT_DIST = 12

local edit = {}

edit.tool = select_rect

edit.name = "edit"

function edit:mousepressed(canvas)
	local mx, my = canvas:get_mouse()

	-- Check if click on note
	local closest, closest_ch = canvas:find_closest_note(mx, my, NOTE_EDIT_DIST)
	local closest_end, closest_ch_end = canvas:find_closest_end(mx, my, DRAG_END_DIST)

	self.select_note = closest
	local select_ch = closest_ch

	if modifier_keys.alt then
		self.tool = adjust_velocity
	elseif closest_end then
		self.tool = drag_end

		self.select_note = closest_end
		select_ch = closest_ch_end
	elseif closest then
		self.tool = drag

		if modifier_keys.ctrl then
			drag.mode = "clone"
		end
	end

	-- If not part of selection already, change selection to just the note we clicked
	if self.select_note and not selection.mask[self.select_note] then
		local mask = { [self.select_note] = true }
		if modifier_keys.shift then
			selection.add(mask)
		else
			selection.set(mask)
		end
		selection.ch_index = select_ch
	end

	self.tool:mousepressed(canvas)
end

function edit:mousedown(canvas)
	self.tool:mousedown(canvas)
end

function edit:mousereleased(canvas)
	local did_edit = self.tool:mousereleased(canvas)
	if not did_edit and modifier_keys.ctrl and selection.mask[self.select_note] then
		selection.subtract({ [self.select_note] = true })
	end

	self.tool = select_rect
end

function edit:update(canvas)
	local mx, my = canvas:get_mouse()
	local closest, _ = canvas:find_closest_note(mx, my, NOTE_EDIT_DIST)
	local closest_end, _ = canvas:find_closest_end(mx, my, DRAG_END_DIST)

	if modifier_keys.alt and not modifier_keys.ctrl then
		mouse:set_cursor("h")
	elseif closest_end or self.tool == drag_end then
		mouse:set_cursor("v")
	elseif closest or self.tool == drag then
		mouse:set_cursor("move")
	end
end

function edit:draw(canvas)
	self.tool:draw(canvas)
end

return edit
