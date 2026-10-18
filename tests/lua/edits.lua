-- Tool edits that are in progress while the project changes underneath them (undo, tuning changes).
-- Drives the real canvas and tools, without drawing.
-- cargo run -- --test-run tests/lua/edits.lua

local Canvas = require("views/canvas")
local Ui = require("ui/ui")
local edit = require("tools/edit")
local pen = require("tools/pen")
local presets = require("default.tuning_presets")
local tuning = require("tuning")

-- icons need the graphics context, the canvas only stores them
tessera.icon = tessera.icon or {}

local canvas

local function setup()
	project = {
		settings = { snap_pitch = 2, snap_time = 4, preview_notes = false },
		channels = { { notes = {}, visible = true } },
		time = { { 0, 2 } },
	}
	command.stack = {}
	command.index = 0
	tuning.system = nil
	tuning.center = nil
	tuning.load_project()
	selection.deselect()
	selection.ch_index = 1

	canvas = Canvas.new()
	-- so canvas:get_mouse() is the mouse position
	canvas.box = { x = -Ui.BORDER_SIZE, y = -(Ui.HEADER + Ui.BORDER_SIZE), focus = true }
	canvas.w, canvas.h = 1000, 1000
	mouse:load()
end

local function press(x, y)
	mouse.x, mouse.y = x, y
	mouse.button = 1
	mouse.is_down = true
	canvas:mousepressed()
end

local function move(x, y)
	mouse.x, mouse.y = x, y
	mouse.drag = true
	canvas:update()
end

local function release()
	canvas:mousereleased()
	mouse.is_down = false
	mouse.button = false
	mouse.drag = false
end

local function undo()
	command.undo()
	tuning.validate()
end

local function set_tuning(key)
	local def = util.clone(presets.tunings[key])
	def.notation = tessera.tuning.notations(def)[1].choice
	local c = command.SetTuning.new(def)
	assert(c:run())
	command.register(c)
end

-- a note at C4, and where it is on screen
local function add_note()
	local note = { interval = tuning.new_interval(), time = 1, vel = 0.5, verts = { { 0, 0, 0 }, { 1, 0, 0 } } }
	command.run_and_register(command.NoteAdd.new({ { note } }))
	return note, canvas.transform:time(1), canvas.transform:pitch(60)
end

local tests = {}

-- The original bug: undo while drawing a note reverted the tuning under the pen.
tests["pen, undo tuning change"] = function()
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

tests["drag, undo tuning change"] = function()
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
tests["grab, apply tuning"] = function()
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
tests["tab while dragging"] = function()
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

local names = {}
for name in pairs(tests) do
	table.insert(names, name)
end
table.sort(names)

local failures = 0
for _, name in ipairs(names) do
	setup()
	local ok, err = pcall(tests[name])
	if ok then
		ok, err = pcall(tuning.validate)
	end
	if ok then
		print("ok    " .. name)
	else
		failures = failures + 1
		print("FAIL  " .. name .. ": " .. tostring(err))
	end
end

return failures
