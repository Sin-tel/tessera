local Ui = require("ui/ui")
local View = require("view")
local tuning = require("tuning")
local tuning_presets = require("default.tuning_presets")
local widgets = require("ui/widgets")

local ProjectSettings = View.derive("Project Settings")
ProjectSettings.__index = ProjectSettings

local TUNING_KEYS = {
	"meantone",
	"ji_5",
	"marvel",
	"pele_7",
	"meantone_19et",
	"meantone_31et",
	"et_41",
}

function ProjectSettings.new()
	local self = setmetatable({}, ProjectSettings)

	self.ui = Ui.new(self)
	self.ui.layout.h = Ui.scale(32)
	self.ui.layout:padding(6)
	self.indent = Ui.scale(32)

	local name_list = {}
	for i, k in ipairs(TUNING_KEYS) do
		local name = tuning_presets[k].name
		assert(name)
		name_list[i] = name
	end

	self.tuning_key = 1
	self.select_tuning = widgets.Dropdown.new(self, "tuning_key", { list = name_list, no_undo = true })

	return self
end

function ProjectSettings:update()
	local key = util.find(TUNING_KEYS, project.settings.tuning_key)
	if key then
		self.tuning_key = key
	end

	local s = Ui.scale(64)
	local lw = math.min(800, self.w - s)
	local x = Ui.scale(64)
	local y = Ui.scale(24)

	local c1 = self.indent
	local c2 = 0.3 * (lw - c1)
	local c3 = 0.4 * (lw - c1)

	self.ui:start_frame(x, y)

	self.ui.layout:col(c1 + c2)
	self.ui:label("Tuning system")
	self.ui.layout:col(c3)
	if self.select_tuning:update(self.ui) then
		tuning.load(TUNING_KEYS[self.tuning_key])
	end
	self.ui.layout:new_row()

	self.ui:end_frame()
end

function ProjectSettings:draw()
	self.ui:draw()
end

function ProjectSettings:mousepressed()
	--
end

return ProjectSettings
