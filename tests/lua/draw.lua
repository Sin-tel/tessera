-- Drawing notes: the freehand tool.
-- cargo run -- --test-run tests/lua/draw.lua

local freehand = require("tools/freehand")
local grab = require("tools/grab")
local smooth = require("tools/smooth")
local tuning = require("tuning")

local h = dofile("tests/lua/lib/harness.lua")
local press, move, release, undo = h.press, h.move, h.release, h.undo

local tests = {}

local function notes()
	return project.channels[1].notes
end

-- pitch of a note at absolute time t, linear between verts
local function pitch_at(note, t)
	local base = tuning.get_pitch(note.interval)
	local v = note.verts
	t = t - note.time
	for i = 2, #v do
		if t <= v[i][1] then
			local s = (t - v[i - 1][1]) / (v[i][1] - v[i - 1][1])
			return base + util.lerp(v[i - 1][2], v[i][2], s)
		end
	end
	return base + v[#v][2]
end

local function assert_well_formed(note)
	assert(note.verts[1][1] == 0, "first vert not at the note start")
	for i = 2, #note.verts do
		assert(note.verts[i][1] > note.verts[i - 1][1], "verts not increasing at " .. i)
	end
end

-- Draw through screen points without the stabilizer, so the line goes exactly through them.
local function draw(canvas, path)
	local radius = freehand.radius
	freehand.radius = 0
	canvas:select_tool(freehand)
	press(path[1][1], path[1][2])
	for i = 2, #path do
		move(path[i][1], path[i][2])
	end
	release()
	freehand.radius = radius
end

local function wave(x)
	return 380 + 40 * math.sin(x / 50)
end

tests["draws a note along the line"] = function(canvas)
	local path = {}
	for x = 200, 600, 4 do
		table.insert(path, { x, wave(x) })
	end
	draw(canvas, path)

	assert(#notes() == 1, "expected one note, got " .. #notes())
	local note = notes()[1]
	assert_well_formed(note)
	assert(#note.verts < #path / 2, "line was not simplified")
	assert(math.abs(note.time - canvas.transform:time_inv(200)) < 1e-9, "wrong start time")
	assert(tuning.eq(note.interval, tuning.snap(canvas.transform:pitch_inv(wave(200)))), "start not snapped")

	local tol = 2.5 / -canvas.transform.sy
	for _, pt in ipairs(path) do
		local want = canvas.transform:pitch_inv(pt[2])
		local got = pitch_at(note, canvas.transform:time_inv(pt[1]))
		assert(math.abs(got - want) < tol, ("off the line at x=%d by %.3f"):format(pt[1], got - want))
	end
	for _, v in ipairs(note.verts) do
		assert(v[3] == 0.5, "pressure not set")
	end
end

tests["straight line is two verts"] = function(canvas)
	local path = {}
	for x = 200, 600, 5 do
		table.insert(path, { x, 380 })
	end
	draw(canvas, path)
	assert(#notes()[1].verts == 2)
end

tests["drawing back replaces the line"] = function(canvas)
	local path = {}
	for x = 200, 600, 5 do
		table.insert(path, { x, 380 })
	end
	for x = 600, 400, -5 do
		table.insert(path, { x, 320 })
	end
	draw(canvas, path)

	local note = notes()[1]
	assert_well_formed(note)
	local t_end = note.time + note.verts[#note.verts][1]
	assert(math.abs(t_end - canvas.transform:time_inv(600)) < 1e-9, "end moved")
	local high = canvas.transform:pitch_inv(320)
	local low = canvas.transform:pitch_inv(380)
	assert(math.abs(pitch_at(note, canvas.transform:time_inv(500)) - high) < 0.01, "not replaced")
	assert(math.abs(pitch_at(note, canvas.transform:time_inv(300)) - low) < 0.01, "start changed")
end

tests["drawing left of the start"] = function(canvas)
	local path = {}
	for x = 400, 200, -5 do
		table.insert(path, { x, 380 + (400 - x) * 0.2 })
	end
	draw(canvas, path)
	local note = notes()[1]
	assert_well_formed(note)
	assert(math.abs(note.time - canvas.transform:time_inv(200)) < 1e-9, "note does not start at the left end")
end

tests["stabilizer trails the mouse"] = function(canvas)
	canvas:select_tool(freehand)
	press(200, 380)
	move(260, 380)
	move(300, 380)
	release()
	local note = notes()[1]
	local t_end = note.time + note.verts[#note.verts][1]
	assert(math.abs(canvas.transform:time(t_end) - (300 - freehand.radius)) < 1e-6, "pen not trailing")
end

tests["click without moving adds nothing"] = function(canvas)
	canvas:select_tool(freehand)
	local steps = command.index
	press(300, 380)
	move(300, 380)
	release()
	assert(#notes() == 0 and command.index == steps)
end

tests["vertical only adds nothing"] = function(canvas)
	draw(canvas, { { 300, 380 }, { 300, 340 }, { 300, 300 } })
	assert(#notes() == 0)
end

tests["undo removes the note"] = function(canvas)
	draw(canvas, { { 200, 380 }, { 300, 360 }, { 400, 380 } })
	assert(#notes() == 1)
	undo()
	assert(#notes() == 0)
end

tests["locked channel"] = function(canvas)
	project.channels[1].lock = true
	draw(canvas, { { 200, 380 }, { 300, 360 }, { 400, 380 } })
	assert(#notes() == 0)
end

-- A drawn note can be sculpted right after.
tests["sculpt a drawn note"] = function(canvas)
	for _, tool in ipairs({ smooth, grab }) do
		local path = {}
		for x = 200, 600, 4 do
			table.insert(path, { x, wave(x) })
		end
		draw(canvas, path)
		local note = notes()[#notes()]
		local before = util.clone(note.verts)

		canvas:select_tool(tool)
		press(400, wave(400))
		for i = 1, 10 do
			move(400 + i, wave(400) - 3 * i)
		end
		release()
		assert(not util.deep_eq(note.verts, before), tool.name .. " did not change the drawn note")
		undo()
		undo()
	end
end

return h.run(tests)
