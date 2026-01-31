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

	local temperament_name_list = {}
	for i, k in ipairs(tuning.temperaments) do
		local name = tuning_presets[k].name
		assert(name)
		temperament_name_list[i] = name
	end

	local et_name_list = {}
	for i, k in ipairs(tuning.ets) do
		local name = tuning_presets[k].name
		assert(name)
		et_name_list[i] = name
	end

	self.tuning_mode = 1
	self.temperament_index = 1
	self.et_index = util.find(tuning.ets, "et_31")
	self.notation_index = 1
	self.select_tuning_mode = widgets.Selector.new(
		self,
		"tuning_mode",
		{ list = { "Temperament", "Equal", "Just Intonation" }, no_undo = true }
	)

	self.select_temperament =
		widgets.Dropdown.new(self, "temperament_index", { list = temperament_name_list, arrows = true, no_undo = true })
	self.select_et = widgets.Dropdown.new(self, "et_index", { list = et_name_list, arrows = true, no_undo = true })

	self.select_ji_notation =
		widgets.Selector.new(self, "notation_index", { list = { "HEJI", "Johnston" }, no_undo = true })

	return self
end

function ProjectSettings:set_notation()
	local mode = tuning.modes[self.tuning_mode]

	if mode == "ji" then
		if self.notation_index == 1 then
			project.settings.notation_style = "heji"
		else
			project.settings.notation_style = "johnston"
		end
	else
		project.settings.notation_style = "ups"
	end
	print("notation:", project.settings.notation_style)
end

function ProjectSettings:update()
	-- local index = util.find(tuning.systems, project.settings.tuning_key)
	-- if index then
	-- 	self.tuning_index = index
	-- end
	-- index = util.find(tuning.notation_styles, project.settings.notation_style)
	-- if index then
	-- 	self.notation_index = index
	-- end

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

	if self.select_tuning_mode:update(self.ui) then
		self:set_notation()
		local mode = tuning.modes[self.tuning_mode]
		if mode == "ji" then
			tuning.load("ji_11")
		end
	end
	-- 	self:update_category()
	-- 	-- tuning.load(tuning.systems[self.tuning_index])
	-- end

	local mode = tuning.modes[self.tuning_mode]
	if mode == "temperament" then
		self.ui.layout:new_row()
		self.ui.layout:col(c1)
		self.ui.layout:col(c2)
		-- self.ui:label("Temperament")
		self.ui.layout:col(c3)
		if self.select_temperament:update(self.ui) then
			tuning.load(tuning.temperaments[self.temperament_index])
			self:set_notation()
		end
	elseif mode == "equal" then
		self.ui.layout:new_row()
		self.ui.layout:col(c1)
		self.ui.layout:col(c2)
		-- self.ui:label("ET")
		self.ui.layout:col(c3)
		if self.select_et:update(self.ui) then
			tuning.load(tuning.ets[self.et_index])
			self:set_notation()
		end
	elseif mode == "ji" then
		self.ui.layout:new_row()
		self.ui.layout:col(c1)
		self.ui.layout:col(c2)
		self.ui:label("Notation")
		self.ui.layout:col(c3)
		if self.select_ji_notation:update(self.ui) then
			self:set_notation()
		end
	else
		assert(false, "unreachable")
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
