local build = require("build")
local save = require("save")

local file = {}

local dialog_pending

local load_last_save = true
local overwrite_check = true

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
				build.new_project()
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
