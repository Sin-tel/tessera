local Ui = require("ui/ui")
local View = require("view")
local tuning = require("tuning")
local tuning_presets = require("default.tuning_presets")
local widgets = require("ui/widgets")

local ProjectSettings = View.derive("Project Settings")
ProjectSettings.__index = ProjectSettings

function ProjectSettings.new()
	local self = setmetatable({}, ProjectSettings)

	self.ui = Ui.new(self)
	self.ui.layout.h = Ui.scale(32)
	self.ui.layout:padding(6)
	self.indent = Ui.scale(32)

	local name_list = {}
	for i, k in ipairs(tuning.systems) do
		local name = tuning_presets[k].name
		assert(name)
		name_list[i] = name
	end

	self.tuning_index = 1
	self.notation_index = 1
	self.select_tuning = widgets.Dropdown.new(self, "tuning_index", { list = name_list, no_undo = true })
	self.select_accidentals =
		widgets.Selector.new(self, "notation_index", { list = { "ups/downs", "HEJI", "Johnston" }, no_undo = true })

	return self
end

function ProjectSettings:update()
	local index = util.find(tuning.systems, project.settings.tuning_key)
	if index then
		self.tuning_index = index
	end
	index = util.find(tuning.notation_styles, project.settings.notation_style)
	if index then
		self.notation_index = index
	end

	tessera.graphics.set_font_main()

	local x = Ui.scale(64)
	local lw = math.min(Ui.scale(600), self.w - 2 * x)
	local y = Ui.scale(24)

	local c1 = self.indent
	local c2 = 0.3 * (lw - c1)
	local c3 = 0.7 * (lw - c1)

	self.ui:start_frame(x, y)

	self.ui.layout:new_row()
	self.ui.layout:col(lw)
	self.ui:label("Tuning settings")

	self.ui:background(theme.bg_nested)

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("System")
	self.ui.layout:col(c3)
	if self.select_tuning:update(self.ui) then
		tuning.load(tuning.systems[self.tuning_index])
	end
	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Style")
	self.ui.layout:col(c3)
	if self.select_accidentals:update(self.ui) then
		project.settings.notation_style = tuning.notation_styles[self.notation_index]
	end

	self.ui:end_frame()
end

function ProjectSettings:draw()
	self.ui:draw()
end

function ProjectSettings:mousepressed()
	--
end

return ProjectSettings
