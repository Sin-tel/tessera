-- Shared setup for test scripts that drive the real canvas and tools, without drawing.
-- Used as `local h = dofile("tests/lua/lib/harness.lua")`. Lives in lib/ so CI doesn't run it as a test.

local Canvas = require("views/canvas")
local Ui = require("ui/ui")
local presets = require("default.tuning_presets")
local tuning = require("tuning")

local h = {}

-- icons need the graphics context, the canvas only stores them
tessera.icon = tessera.icon or {}

-- A fresh project with one channel, default tuning and nothing selected, and a new canvas.
function h.setup()
	project = {
		settings = { snap_pitch = 2, snap_time = 4, preview_notes = false },
		channels = { { notes = {}, visible = true } },
		time = { { 0, 2 } },
	}
	command.stack = {}
	command.index = 0
	tuning.system = nil
	tuning.load_project()
	selection.deselect()
	selection.ch_index = 1
	-- a failed test can leave them on
	for key in pairs(modifier_keys) do
		modifier_keys[key] = false
	end

	h.canvas = Canvas.new()
	-- so canvas:get_mouse() is the mouse position
	h.canvas.box = { x = -Ui.BORDER_SIZE, y = -(Ui.HEADER + Ui.BORDER_SIZE), focus = true }
	h.canvas.w, h.canvas.h = 1000, 1000
	mouse:load()
end

function h.press(x, y)
	mouse.x, mouse.y = x, y
	mouse.button = 1
	mouse.is_down = true
	h.canvas:mousepressed()
end

function h.move(x, y)
	mouse.x, mouse.y = x, y
	mouse.drag = true
	h.canvas:update()
end

function h.release()
	h.canvas:mousereleased()
	mouse.is_down = false
	mouse.button = false
	mouse.drag = false
end

function h.undo()
	command.undo()
	tuning.validate()
end

-- Apply a preset tuning with its recommended notation, as an undoable command.
function h.set_tuning(key)
	local def = util.clone(presets.tunings[key])
	for _, option in ipairs(tessera.tuning.notations(def)) do
		if option.recommended then
			def.notation = option.choice
			break
		end
	end
	local c = command.SetTuning.new(def)
	assert(c:run())
	command.register(c)
end

-- a note at C4, and where it is on screen
function h.add_note()
	local note = { interval = tuning.new_interval(), time = 1, vel = 0.5, verts = { { 0, 0, 0 }, { 1, 0, 0 } } }
	command.run_and_register(command.NoteAdd.new({ { note } }))
	return note, h.canvas.transform:time(1), h.canvas.transform:pitch(60)
end

-- Run `tests` (name -> function(canvas)) in name order, each on a fresh setup.
-- Checks that the project is consistent with the tuning after each one. Returns the number of failures.
function h.run(tests)
	local names = {}
	for name in pairs(tests) do
		table.insert(names, name)
	end
	table.sort(names)

	local failures = 0
	for _, name in ipairs(names) do
		h.setup()
		local ok, err = pcall(tests[name], h.canvas)
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
end

return h
