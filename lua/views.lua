local View = require("view")

local views = {}

views.default = View:derive("Empty")
views.channel = require("views/channelview")
views.panner = require("views/pannerview")
views.parameter = require("views/parameterview")
views.song = require("views/songview")
views.testpad = require("views/testpadview")
views.scope = require("views/scopeview")
views.ui_test = require("views/uitestview")

return views
