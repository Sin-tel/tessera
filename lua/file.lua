local build = require("build")
local log = require("log")
local save = require("save")

local file = {}

local dialog_pending

local load_last_save = true
local overwrite_check = true

-- This file exists while the app runs, deleted on clean exit.
-- If it's still there on startup, the last session crashed.
-- Contains the path of the recovery save if it was written.
local running_marker = "out/running"
local marker_written = false

-- since file dialogs are spawned on new threads, we need to check the results here
function file.poll_dialogs()
	if dialog_pending then
		local f = tessera.dialog_poll()
		if f then
			if dialog_pending == "save" then
				save.write(f)
				dialog_pending = nil
				overwrite_check = false
			elseif dialog_pending == "open" then
				-- TODO: undo
				build.empty_project()
				save.read(f)
				dialog_pending = nil
				overwrite_check = false
			end
		end
	end
end

function file.new()
	command.run_and_register(command.NewProject.new())
	overwrite_check = true
end

function file.open()
	if tessera.dialog_open() then
		dialog_pending = "open"
	end
end

-- Call once on startup, before load_last.
function file.check_crash()
	if util.file_exists(running_marker) then
		log.warn("Last session did not exit cleanly, not loading the last save.")
		local recovery = util.readfile(running_marker)
		if recovery ~= "" then
			log.warn('Recovered project saved to "' .. recovery .. '"')
		end
		load_last_save = false
	end
	util.writefile(running_marker, "")
	marker_written = true
end

-- Save the project after a crash, and note it in the marker.
function file.write_recovery()
	if save.write_recovery() and marker_written then
		util.writefile(running_marker, save.recovery_path)
	end
end

function file.clean_exit()
	if marker_written then
		os.remove(running_marker)
	end
end

function file.load_last()
	local success = false
	if load_last_save then
		local f = save.get_save_location()
		if f then
			success = save.read(f)
		end
	end

	if success then
		overwrite_check = false
	end
	return success
end

function file.save()
	local filename = save.get_save_location()
	if overwrite_check or not filename then
		file.save_as()
	else
		save.write(filename)
	end
end

function file.save_as()
	if tessera.dialog_save("my_project") then
		dialog_pending = "save"
	end
end

return file
