local build = require("build")
local save = require("save")

local file = {}

local dialog_pending

-- since file dialogs are spawned on new threads, we need to check the results here
function file.poll_dialogs()
	if dialog_pending then
		local f = tessera.dialog_poll()
		if f then
			if dialog_pending == "save" then
				save.write(f)
				save.set_save_location(f)
				dialog_pending = nil
			elseif dialog_pending == "open" then
				-- TODO: undo
				build.new_project()
				save.read(f)
				save.set_save_location(f)
				dialog_pending = nil
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

function file.save()
	save.write(save.last_save_location)
end

function file.save_as()
	if tessera.dialog_save("my_project") then
		dialog_pending = "save"
	end
end

return file
