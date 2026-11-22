-- Brush tools and the curve operations under them.
-- cargo run -- --test-run tests/lua/brush.lua

local curve = require("curve")
local grab = require("tools/grab")
local smooth = require("tools/smooth")
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

local function roughness(verts, column)
	local sum = 0
	for i = 2, #verts - 1 do
		sum = sum + math.abs(verts[i - 1][column] + verts[i + 1][column] - 2 * verts[i][column])
	end
	return sum
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

tests["smooth flattens and keeps the ends"] = function(canvas)
	canvas:select_tool(smooth)
	local note = add_note(zigzag(41, constant(curve.RESOLUTION)))
	local before = util.clone(note.verts)
	local n = #before

	stroke(canvas, 1.2, 60, 30)

	assert_well_formed(note)
	local m = #note.verts
	assert(note.time == 1, "note start moved")
	assert(note.verts[m][1] == before[n][1], "note end moved")
	assert(note.verts[1][3] == before[1][3] and note.verts[m][3] == before[n][3], "end pressure changed")
	assert(roughness(note.verts, 2) < 0.5 * roughness(before, 2), "pitch not smoothed")
	assert(roughness(note.verts, 3) < 0.5 * roughness(before, 3), "pressure not smoothed")
end

tests["smooth pulls the end pitch"] = function(canvas)
	canvas:select_tool(smooth)
	local note = add_note({ { 0, 2, 0.5 }, { 0.1, 0, 0.5 }, { 0.2, 0, 0.5 } })
	stroke(canvas, 1, 62, 1)
	assert(note.verts[1][2] < 2 and note.verts[1][2] > 0, "first vert pitch did not follow")
	assert(note.verts[1][1] == 0 and note.time == 1, "first vert moved in time")
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

-- Subdivided while editing, simplified after.
tests["touched notes are resampled at a fixed resolution"] = function(canvas)
	canvas:select_tool(smooth)
	local dense = {}
	for i = 0, 250 do
		dense[i + 1] = { i * 0.008, 0, 0.5 }
	end
	local recorded = add_note(dense)
	local pen = add_note({ { 0, 0, 0.5 }, { 3, 0, 0.5 } }, 1)
	selection.set({ [recorded] = true, [pen] = true })

	-- zoomed out, so nothing depends on zoom
	canvas.transform.sx = canvas.transform.sx / 8
	local x, y = canvas.transform:time(2), canvas.transform:pitch(60)
	press(x, y)
	move(x, y)
	for _, note in ipairs({ recorded, pen }) do
		assert_well_formed(note)
		for i = 2, #note.verts do
			local gap = note.verts[i][1] - note.verts[i - 1][1]
			assert(gap < 1.5 * curve.RESOLUTION, "gap of " .. gap)
		end
	end
	release()
	-- both flat, so nothing left between the ends
	assert(#recorded.verts == 2 and #pen.verts == 2, ("%d and %d verts left"):format(#recorded.verts, #pen.verts))
end

tests["radius keys"] = function(canvas)
	canvas:select_tool(smooth)
	local radius = smooth.radius
	assert(canvas:keypressed("["))
	assert(smooth.radius < radius)
	assert(canvas:keypressed("]"))
	smooth.radius = radius
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

local function max_pitch(verts)
	local m = -math.huge
	for _, v in ipairs(verts) do
		m = math.max(m, v[2])
	end
	return m
end

tests["grab follows the mouse near the press"] = function(canvas)
	canvas:select_tool(grab)
	local note = add_note({ { 0, 0, 0.5 }, { 6, 0, 0.5 } })
	local steps = command.index

	drag(canvas, 4, 60, 0, 2)
	assert_well_formed(note)
	assert(math.abs(max_pitch(note.verts) - 2) < 0.1, "peak at " .. max_pitch(note.verts))
	local n = #note.verts
	assert(math.abs(note.verts[1][2]) < 1e-6 and math.abs(note.verts[n][2]) < 1e-6, "far ends moved")
	assert(note.time == 1 and note.verts[n][1] == 6, "pitch drag moved in time")
	assert(command.index == steps + 1, "grab is not one undo step")
	undo()
	assert(util.deep_eq(note.verts, { { 0, 0, 0.5 }, { 6, 0, 0.5 } }), "undo did not restore the note")
end

tests["grab far in time never crosses verts"] = function(canvas)
	canvas:select_tool(grab)
	local note = add_note(zigzag(61, constant(0.1)))
	for _, dt in ipairs({ 3, -3, 10, -10 }) do
		drag(canvas, 4, 60, dt, 0)
		assert_well_formed(note)
	end
end

-- Weights depend on time only, so a wide vibrato isn't skewed by which verts are level with the mouse.
tests["grab keeps vibrato spacing"] = function(canvas)
	canvas:select_tool(grab)
	local verts = zigzag(61, constant(curve.RESOLUTION))
	for i, v in ipairs(verts) do
		v[2] = 4 * math.sin(i * 0.7)
	end
	local note = add_note(verts)
	local before = util.clone(note.verts)
	drag(canvas, 1.3, 60, 1, 0)
	assert_well_formed(note)
	assert(#note.verts == #before, "resampled")
	for i = 2, #note.verts do
		local ratio = (note.verts[i][1] - note.verts[i - 1][1]) / (before[i][1] - before[i - 1][1])
		assert(ratio > 0.45, ("verts %d-%d bunched to %.2f"):format(i - 1, i, ratio))
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
