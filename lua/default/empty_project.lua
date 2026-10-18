local function empty_project()
	assert(VERSION)
	return {
		channels = {},
		VERSION = util.clone(VERSION),
		name = "Untitled project",

		transport = {
			start_time = 0,
			recording = true,
		},

		settings = {
			preview_notes = true,
			chase = false,
			follow = false,
			snap_time = 3,
			snap_pitch = 1,
			metronome = false,
			relative_note_names = true,
			auto_simplify = false,
			-- settings.tuning is set by tuning.load_project
		},

		-- tuning center over the timeline, interval filled in by tuning.load_project
		center = {
			{ time = 0.0, interval = {} },
		},

		time = {
			{
				0.0,
				2.0,
			},
		},
	}
end

return empty_project
