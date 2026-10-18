-- Tool edits and undo: edits in progress while the project changes underneath them (undo, tuning changes),
-- and what ends up on the undo stack.
-- cargo run -- --test-run tests/lua/edits.lua

local edit = require("tools/edit")
local pen = require("tools/pen")
local presets = require("default.tuning_presets")

local h = dofile("tests/lua/lib/harness.lua")
local press, move, release, undo, set_tuning, add_note = h.press, h.move, h.release, h.undo, h.set_tuning, h.add_note

local tests = {}

-- The original bug: undo while drawing a note reverted the tuning under the pen.
tests["pen, undo tuning change"] = function(canvas)
	set_tuning("ji_7")
	canvas:select_tool(pen)
	press(300, 380)
	move(400, 380)
	-- cancels the note
	undo()
	assert(#project.channels[1].notes == 0, "pen note was not undone")
	undo()
	assert(project.settings.tuning.name == presets.default().name, "tuning was not undone")
	move(450, 380)
	release()
	assert(#project.channels[1].notes == 0, "pen added a note after it was finished")
end

tests["drag, undo tuning change"] = function(canvas)
	local note, x, y = add_note()
	set_tuning("ji_7")
	canvas:select_tool(edit)
	press(x, y)
	move(x + 50, y - 100)
	-- cancels the drag
	undo()
	assert(note.time == 1, "drag was not undone")
	undo()
	move(x + 100, y - 100)
	release()
	assert(note.time == 1, "note moved after the drag was finished")
end

-- Keyboard grab stays active until a click, so the tuning can change in another tab.
tests["grab, apply tuning"] = function(canvas)
	local note, x, y = add_note()
	selection.set({ [note] = true })
	mouse.x, mouse.y = x, y
	canvas:keypressed("g")
	move(x + 50, y - 100)
	set_tuning("ji_7")
	assert(not canvas.tool_active, "grab still active after tuning change")
	move(x + 100, y - 100)
	release()
	undo()
	undo()
	undo()
	assert(#project.channels[1].notes == 0)
end

-- Switching tools mid drag used to drop the drag without registering it.
tests["tab while dragging"] = function(canvas)
	local _, x, y = add_note()
	canvas:select_tool(edit)
	press(x, y)
	move(x + 50, y)
	canvas:keypressed("tab")
	move(x + 100, y)
	release()
	assert(command.stack[command.index].prev_state, "drag was not registered")
	assert(canvas.current_tool == pen, "tool did not switch after the drag")
end

-- Edits that end up changing nothing don't add an undo step.
tests["empty edits"] = function(canvas)
	local note, x, y = add_note()
	local n = command.index
	canvas:select_tool(edit)

	-- dragged and back
	press(x, y)
	move(x + 50, y - 100)
	move(x, y)
	release()
	assert(note.time == 1, "note moved")
	assert(command.index == n, "drag back to the start registered")

	press(x, y)
	move(x + 100, y)
	release()
	assert(command.index == n + 1, "real drag not registered")

	command.register(command.NoteAdd.new({}))
	command.register(command.Change.new(project.settings, "snap_time", project.settings.snap_time))
	assert(command.index == n + 1, "empty command registered")
end

return h.run(tests)
