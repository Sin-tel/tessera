local views = {}

views.Canvas = require("views/canvas")
views.Channels = require("views/channels")
views.ChannelSettings = require("views/channel_settings")
views.Debug = require("views/debug")
views.Empty = require("views/empty")
views.Log = require("views/log")
views.Scope = require("views/scope")
views.Settings = require("views/settings")
views.TestPad = require("views/test_pad")
views.UiTest = require("views/ui_test")

-- it's reflection time
function views.get_class_name(instance)
	for id, class in pairs(views) do
		if type(class) == "table" then
			if getmetatable(instance) == class then
				return id
			end
		end
	end
	return nil
end

return views
