-- Brush tools and the curve operations under them.
-- cargo run -- --test-run tests/lua/brush.lua

local curve = require("curve")
local grab = require("tools/grab")
local smooth = require("tools/smooth")
local smudge = require("tools/smudge")
local tuning = require("tuning")

local h = dofile("tests/lua/lib/harness.lua")
local press, move, release, undo = h.press, h.move, h.release, h.undo

local tests = {}

local function add_note(verts, time)
	local note = { interval = tuning.new_interval(), time = time or 1, vel = 0.5, verts = verts }
	command.run_and_register(command.NoteAdd.new({ { note } }))
	return note
end

-- alternating pitch and pressure, `dt(i)` between verts
local function zigzag(n, dt)
	local verts = {}
	local t = 0
	for i = 1, n do
		verts[i] = { t, i % 2 * 2 - 1, i % 2 * 0.6 + 0.2 }
		t = t + dt(i)
	end
	return verts
end

local function constant(dt)
	return function()
		return dt
	end
end

-- stroke in place at time t, pitch p
local function stroke(canvas, t, p, frames)
	local x, y = canvas.transform:time(t), canvas.transform:pitch(p)
	press(x, y)
	for _ = 1, frames do
		move(x, y)
	end
	release()
end

local function assert_well_formed(note)
	assert(note.verts[1][1] == 0, "first vert moved off the note start")
	for i = 2, #note.verts do
		assert(note.verts[i][1] > note.verts[i - 1][1], "verts crossed at " .. i)
	end
end

tests["smooth never crosses verts"] = function(canvas)
	canvas:select_tool(smooth)
	math.randomseed(1)
	local note = add_note(zigzag(200, function()
		return 0.001 + math.random() ^ 4 * 0.1
	end))
	local radius = smooth.radius
	for i = 1, 40 do
		smooth.radius = 5 + i * 3
		stroke(canvas, 1 + i * 0.1, 60 + (i % 5 - 2), 5)
		assert_well_formed(note)
	end
	smooth.radius = radius
end

tests["undo and redo a stroke"] = function(canvas)
	canvas:select_tool(smooth)
	local note = add_note(zigzag(41, constant(0.05)))
	local before = util.clone(note.verts)
	local steps = command.index

	stroke(canvas, 2, 60, 10)
	local after = util.clone(note.verts)
	assert(command.index == steps + 1, "stroke is not one undo step")

	undo()
	assert(util.deep_eq(note.verts, before), "undo did not restore the note")
	command.redo()
	assert(util.deep_eq(note.verts, after), "redo did not restore the stroke")
end

tests["without a selection only the closest note changes"] = function(canvas)
	canvas:select_tool(smooth)
	local near = add_note(zigzag(41, constant(0.05)))
	local far = add_note(zigzag(41, constant(0.05)))
	far.interval = tuning.mul(tuning.tone, 1)
	local before = util.clone(far.verts)

	stroke(canvas, 2, 59, 10)
	assert(util.deep_eq(far.verts, before), "other note was changed")
	assert(not util.deep_eq(near.verts, zigzag(41, constant(0.05))), "closest note not changed")
end

tests["with a selection only selected notes change"] = function(canvas)
	canvas:select_tool(smooth)
	local unselected = add_note(zigzag(41, constant(0.05)))
	local selected = add_note(zigzag(41, constant(0.05)))
	selected.interval = tuning.mul(tuning.octave, 1)
	selection.set({ [selected] = true })
	local before = util.clone(unselected.verts)

	stroke(canvas, 2, 60, 10)
	assert(util.deep_eq(unselected.verts, before), "unselected note was changed")
end

tests["stroke out of reach does nothing"] = function(canvas)
	canvas:select_tool(smooth)
	local note = add_note(zigzag(41, constant(0.05)))
	local before = util.clone(note.verts)
	local steps = command.index

	stroke(canvas, 8, 60, 10)
	assert(util.deep_eq(note.verts, before), "note changed")
	assert(command.index == steps, "empty stroke registered")
end

-- drag from time t, pitch p by dt, dp
local function drag(canvas, t, p, dt, dp)
	local x, y = canvas.transform:time(t), canvas.transform:pitch(p)
	local dx, dy = dt * canvas.transform.sx, dp * canvas.transform.sy
	press(x, y)
	for i = 1, 5 do
		move(x + dx * i / 5, y + dy * i / 5)
	end
	release()
end

tests["grab far in time never crosses verts"] = function(canvas)
	canvas:select_tool(grab)
	local note = add_note(zigzag(61, constant(0.1)))
	for _, dt in ipairs({ 3, -3, 10, -10 }) do
		drag(canvas, 4, 60, dt, 0)
		assert_well_formed(note)
	end
end

tests["grab the start moves the note"] = function(canvas)
	canvas:select_tool(grab)
	local note = add_note({ { 0, 0, 0.5 }, { 6, 0, 0.5 } })
	drag(canvas, 1, 60, -0.5, 0)
	assert_well_formed(note)
	assert(math.abs(note.time - 0.5) < 1e-6, "note starts at " .. note.time)
	assert(math.abs(note.time + note.verts[#note.verts][1] - 7) < 1e-6, "far end moved")
end

tests["grab click without moving does nothing"] = function(canvas)
	canvas:select_tool(grab)
	local note = add_note({ { 0, 0, 0.5 }, { 6, 0, 0.5 } })
	local steps = command.index
	drag(canvas, 4, 60, 0, 0)
	assert(#note.verts == 2, "note was subdivided")
	assert(command.index == steps, "click registered an edit")
end

-- Shift switches grab to smooth for one stroke, only while shift alone is held.
tests["shift smooths with grab selected"] = function(canvas)
	canvas:select_tool(grab)
	modifier_keys.shift, modifier_keys.any = true, true
	assert(canvas:hover_tool() == smooth)
	modifier_keys.ctrl = true
	assert(canvas:hover_tool() == grab, "shift+ctrl should not switch")
	modifier_keys.ctrl = false

	local note = add_note(zigzag(41, constant(curve.RESOLUTION)))
	local x, y = canvas.transform:time(1.4), canvas.transform:pitch(60)
	press(x, y)
	assert(canvas.current_tool == smooth, "stroke did not use smooth")
	for _ = 1, 30 do
		move(x, y)
	end
	-- letting go of shift mid-stroke doesn't switch back
	modifier_keys.shift, modifier_keys.any = false, false
	move(x, y)
	assert(canvas.current_tool == smooth)
	release()
	assert(canvas.selected_tool == grab and canvas.current_tool == grab)
	assert(note.time == 1, "note moved")
end

-- Fast strokes back and forth over a vibrato, verts must stay in order and the start moves the note.
tests["smudge never crosses verts"] = function(canvas)
	canvas:select_tool(smudge)
	local verts = zigzag(201, constant(curve.RESOLUTION))
	for i, v in ipairs(verts) do
		v[2] = 4 * math.sin(i * 0.7)
	end
	local note = add_note(verts)
	local tf = canvas.transform
	press(tf:time(2), tf:pitch(60))
	for i = 1, 60 do
		move(tf:time(2) + (i % 2 * 2 - 1) * 200, tf:pitch(60 + i % 3))
		assert_well_formed(note)
	end
	release()
	assert_well_formed(note)

	press(tf:time(1), tf:pitch(60))
	move(tf:time(1) - 20, tf:pitch(60))
	release()
	assert(note.time < 1, "start did not move the note")
	assert_well_formed(note)
end

tests["simplify keeps corners and ends"] = function()
	local verts = {}
	for i = 0, 100 do
		verts[i + 1] = { i * 0.01, math.abs(i - 50) * 0.1, 0.5 }
	end
	local out = curve.simplify(verts)
	assert(#out == 3, "expected 3 verts, got " .. #out)
	assert(out[1] == verts[1] and out[2] == verts[51] and out[3] == verts[101])
end

tests["simplify stays within tolerance of pressure"] = function()
	local verts = { { 0, 0, 0 }, { 1, 0, 0.5 }, { 2, 0, 0 } }
	assert(#curve.simplify(verts) == 3, "pressure peak was removed")
end

tests["subdivide doesn't change the curve"] = function()
	local out = curve.subdivide({ { 0, 0, 0 }, { 1, 2, 1 } })
	for i = 2, #out do
		assert(math.abs(out[i][1] - out[i - 1][1] - curve.RESOLUTION) < 0.5 * curve.RESOLUTION)
		assert(math.abs(out[i][2] - 2 * out[i][1]) < 1e-9 and math.abs(out[i][3] - out[i][1]) < 1e-9)
	end
end

return h.run(tests)
