local build = require("build")
local log = require("log")
local save = require("save")

local file = {}

-- current save path, nil for a new project
file.path = nil

-- the last file is reopened on startup
local last_save = "out/last_save"

local dialog_pending

local load_last_save = true

-- This file exists while the app runs, deleted on clean exit.
-- If it's still there on startup, the last session crashed.
-- Contains the path of the recovery save if it was written.
local running_marker = "out/running"
local marker_written = false

function file.set_path(path)
	file.path = path
	if path then
		util.writefile(last_save, path)
	end
end

-- since file dialogs are spawned on new threads, we need to check the results here
function file.poll_dialogs()
	if dialog_pending then
		local f = tessera.dialog_poll()
		if f then
			if dialog_pending == "save" then
				dialog_pending = nil
				save.write(f)
				file.set_path(f)
			elseif dialog_pending == "open" then
				dialog_pending = nil
				file.open_path(f)
			end
		end
	end
end

function file.new()
	command.run_and_register(command.NewProject.new())
end

function file.open()
	if tessera.dialog_open() then
		dialog_pending = "open"
	end
end

function file.open_path(path)
	local new_project = save.read(path)
	if new_project then
		command.run_and_register(command.OpenProject.new(new_project, path))
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
	if not (load_last_save and util.file_exists(last_save)) then
		return false
	end
	local path = util.readfile(last_save)
	local new_project = path and path ~= "" and save.read(path)
	if not new_project then
		return false
	end
	build.load_project(new_project)
	file.set_path(path)
	return true
end

function file.save()
	if file.path then
		save.write(file.path)
	else
		file.save_as()
	end
end

function file.save_as()
	if tessera.dialog_save("my_project") then
		dialog_pending = "save"
	end
end

return file
