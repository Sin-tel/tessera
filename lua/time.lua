-- time / tempo / grid settings

local time = {}

local SNAP_TIMES = { 1, 1 / 4, 1 / 16 }
time.snap_labels = { "1", "1/4", "1/16", "Off" }

function time.get_div(t)
	-- for now we just have a single div
	return project.time[1]
end

function time.to_seconds(t)
	local div = time.get_div(t)
	local t_start = div[1]
	local t_mul = div[2]

	return t_start + t / t_mul
end

function time.from_seconds(t)
	local div = time.get_div(t)
	local t_start = div[1]
	local t_mul = div[2]

	return (t - t_start) * t_mul
end

function time.next(t, dt)
	local t_prev = t
	local t_new = t + dt

	if tessera.audio.ok() then
		local beat_prev = math.ceil(time.from_seconds(t_prev))
		local beat = math.ceil(time.from_seconds(t_new))
		if project.settings.metronome and beat ~= beat_prev then
			tessera.audio.metronome(beat % 4 == 1)
		end
	end

	return t_new
end

function time.snap(t)
	if project.settings.snap_time < 4 then
		local r = SNAP_TIMES[project.settings.snap_time]

		t = time.from_seconds(t)
		t = math.floor(t / r + 0.5) * r
		return time.to_seconds(t)
	end
	return t
end

function time.snap_length(t)
	if project.settings.snap_time < 4 then
		local div = time.get_div(t)
		local t_mul = div[2]
		return SNAP_TIMES[project.settings.snap_time] / t_mul
	end
	return 0
end

function time.get_grid(t0, t1, scale)
	local div = time.get_div(t0)
	local t_start = div[1]
	local t_mul = div[2]

	-- target line density
	local res_target = (80 / scale) * t_mul

	-- round to nearest power of 4: 1/16, 1/4, 1, 4, 16, ...
	local res = 4 ^ math.floor(math.log(res_target, 4) + 0.5)
	-- local res = 1

	local b0 = (t0 - t_start) * t_mul
	local b1 = (t1 - t_start) * t_mul

	local grid_major = {}
	local grid_minor = {}

	for i = math.ceil(b0 / res), math.floor(b1 / res) do
		local t = t_start + (i * res) / t_mul
		if i % 4 == 0 then
			table.insert(grid_major, t)
		else
			table.insert(grid_minor, t)
		end
	end

	return grid_major, grid_minor
end

return time
