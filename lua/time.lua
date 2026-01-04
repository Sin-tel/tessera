-- time / tempo / grid settings

local time = {}

time.snap_times = { 1, 1 / 4, 1 / 16 }
time.snap_labels = { "1", "1/4", "1/16", "Off" }

function time.snap(t)
	if project.settings.snap_time < 4 then
		local r = time.snap_times[project.settings.snap_time]
		return (math.floor(t / r + 0.5) * r)
	end
	return t
end

function time.snap_length()
	if project.settings.snap_time < 4 then
		return time.snap_times[project.settings.snap_time]
	end
	return 0
end

return time
