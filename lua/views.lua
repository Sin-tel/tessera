local View = require("view")

local views = {}

views.Default = View.derive("Empty")
views.ChannelSettings = require("views/channel_settings")
views.Channels = require("views/channels")
views.Scope = require("views/scope")
views.Canvas = require("views/canvas")
views.TestPad = require("views/test_pad")
views.UiTest = require("views/ui_test")
views.Debug = require("views/debug")

return views
