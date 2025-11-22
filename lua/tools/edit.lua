local adjust_velocity = require("tools/adjust_velocity")
local drag = require("tools/drag")
local drag_end = require("tools/drag_end")
local select_rect = require("tools/select_rect")

local edit = {}

edit.tool = select_rect

function edit:mousepressed(canvas)
	local mx, my = canvas:getMouse()

	self.tool = select_rect

	-- Check if click on note
	local closest, closest_ch = canvas:find_closest_note(mx, my, 24)
	local closest_end, closest_ch_end = canvas:find_closest_end(mx, my, 24)

	local select_note = closest
	local select_ch = closest_ch

	if modifier_keys.alt then
		self.tool = adjust_velocity
	elseif closest_end then
		self.tool = drag_end

		select_note = closest_end
		select_ch = closest_ch_end
	elseif closest then
		self.tool = drag
	end

	-- If not part of selection already, change selection to just the note we clicked
	if select_note and not selection.mask[select_note] then
		selection.setNormal({ [select_note] = true })
		selection.ch_index = select_ch
	end

	self.tool:mousepressed(canvas)
end

function edit:mousedown(canvas)
	self.tool:mousedown(canvas)
end

function edit:mousereleased(canvas)
	self.tool:mousereleased(canvas)
end

function edit:draw(canvas)
	self.tool:draw(canvas)
end

return edit
