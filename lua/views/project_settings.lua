local Notation = require("notation")
local Ui = require("ui/ui")
local View = require("view")
local tuning = require("tuning")
local tuning_presets = require("default.tuning_presets")
local widgets = require("ui/widgets")

local ProjectSettings = View.derive("Project Settings")
ProjectSettings.__index = ProjectSettings

-- One choice of notation for the current tuning, and how it writes the primes.
local NotationRow = {}
NotationRow.__index = NotationRow

local current_def
local current_info

-- Compare two Definition or NotationInfo's for equality,
-- given they are derived from the same temperament.
local function cmp_info(a, b)
	return a.n_accidentals == b.n_accidentals and a.half_sharp == b.half_sharp
end

function NotationRow.new(info)
	local self = setmetatable({}, NotationRow)

	self.info = info

	local notation = Notation.new(info)

	self.title = "Plain"
	if info.n_accidentals == 1 then
		self.title = "Single"
	elseif info.n_accidentals > 0 then
		self.title = tostring(info.n_accidentals) .. " accidentals"
	end
	if info.half_sharp then
		self.title = "Neutral"
	end

	self.primes = {}
	for i, s in ipairs(info.spellings) do
		self.primes[i] = {
			ratio = s.ratio[1] .. "/" .. s.ratio[2],
			name = notation:name(s.note),
		}
	end

	return self
end

-- The notation that is in use.
function NotationRow:is_current()
	return cmp_info(current_info, self.info)
end

function NotationRow:update(ui)
	local x, y, w, h = ui:next()
	ui:hitbox(self, x, y, w, h)
	ui:push_draw(self.draw, { self, ui, x, y, w, h })
	return ui.clicked == self
end

function NotationRow:draw(ui, x, y, w, h)
	local indent = 16
	local title_w = 0.32 * w
	if self:is_current() then
		tessera.graphics.set_color(theme.widget)
		tessera.graphics.rectangle("fill", x + Ui.PAD, y, title_w - 2 * Ui.PAD, h, Ui.CORNER_RADIUS)
	elseif ui.active == self then
		tessera.graphics.set_color(theme.widget_press)
		tessera.graphics.rectangle("fill", x + Ui.PAD, y, title_w - 2 * Ui.PAD, h, Ui.CORNER_RADIUS)
	elseif ui.hover == self and ui.active ~= self then
		tessera.graphics.set_color(theme.line_hover)
		tessera.graphics.rectangle("line", x + Ui.PAD, y, title_w - 2 * Ui.PAD, h, Ui.CORNER_RADIUS)
	end

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(self.title, x + indent, y, title_w, h, tessera.graphics.ALIGN_LEFT)

	-- ratio of the prime, and how it is written
	local col_w = (w - title_w) / math.max(1, #self.primes)
	for i, p in ipairs(self.primes) do
		local cx = x + title_w + (i - 1) * col_w
		tessera.graphics.set_font_notes()
		tessera.graphics.set_color(theme.ui_text)
		tessera.graphics.label(p.ratio .. ": " .. p.name, cx, y, col_w, h, tessera.graphics.ALIGN_LEFT)
		tessera.graphics.set_font_main()
	end
end

function ProjectSettings.new()
	local self = setmetatable({}, ProjectSettings)

	self.ui = Ui.new(self)
	self.ui.layout.h = Ui.scale(32)
	self.ui.layout:padding(6)
	self.indent = Ui.scale(32)

	local name_list = {}
	for i, k in ipairs(tuning.presets) do
		local name = tuning_presets.tunings[k].name
		assert(name)
		name_list[i] = name
	end

	self.tuning_mode = 1
	self.select_tuning_mode = widgets.Selector.new(
		self,
		"tuning_mode",
		{ list = { "Temperament", "Equal", "Just Intonation" }, no_undo = true }
	)

	self.preset_index = 1
	self.select_preset = widgets.Dropdown.new(self, "preset_index", { list = name_list, arrows = true, no_undo = true })

	self.apply = widgets.Button.new("Apply")

	self.rows = {}

	return self
end

local function set_current(info)
	current_def.n_accidentals = info.n_accidentals
	current_def.half_sharp = info.half_sharp
	current_info = info
end

-- Show the preset that is active, if there is one.
function ProjectSettings:sync_preset()
	if current_def then
		return
	end

	current_def = project.settings.tuning
	assert(current_def)
	for i, k in ipairs(tuning.presets) do
		if tuning_presets.tunings[k].name == current_def.name then
			self.preset_index = i
		end
	end

	self:update_rows()

	-- find right row
	for _, row in ipairs(self.rows) do
		if cmp_info(current_def, row.info) then
			set_current(row.info)
		end
	end
end

function ProjectSettings:update_rows()
	local def = current_def
	if not def then
		return
	end

	self.rows = {}
	self.error = nil
	local infos = tessera.tuning.notations(def)

	local found_recommended = false

	for _, info in ipairs(infos) do
		-- set the first recommended one as current
		if not found_recommended and info.recommended then
			found_recommended = true
			set_current(info)
		end
		table.insert(self.rows, NotationRow.new(info, false))
	end
end

-- Changing this can change how notes are written, so it needs to be undoable
-- local function set_tuning(def)
-- 	tuning.set(def)
-- 	local c = command.SetTuning.new(def)
-- 	if c:run() then
-- 		command.register(c)
-- 	end
-- end

function ProjectSettings:update()
	self:sync_preset()

	tessera.graphics.set_font_main()

	local x = Ui.scale(64)
	local lw = math.min(Ui.scale(600), self.w - 2 * x)
	local y = Ui.scale(24)

	local c1 = self.indent
	local c2 = 0.3 * (lw - c1)
	local c3 = 0.7 * (lw - c1)

	self.ui:start_frame(x, y)

	-- self.ui.layout:new_row()
	self.ui.layout:col(lw)
	self.ui:label("Tuning system")

	self.ui:background(theme.bg_nested)

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Category")
	self.ui.layout:col(c3)
	if self.select_tuning_mode:update(self.ui) then
		-- TODO
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Tuning")
	self.ui.layout:col(c3)

	if self.select_preset:update(self.ui) then
		-- clicked on a preset, update definition and rows
		current_def = tuning_presets.tunings[tuning.presets[self.preset_index]]
		self:update_rows()
	end

	self.ui:background(theme.background)
	self.ui:label("Notation")
	self.ui:background(theme.bg_nested)

	if self.error then
		self.ui.layout:col(c3)
		self.ui:label("Not available")
	end

	for _, row in ipairs(self.rows) do
		self.ui.layout:col(c1)
		self.ui.layout:col(lw - c1)
		if row:update(self.ui) and not row:is_current() then
			-- clicked on a notation, update definition and current info
			set_current(row.info)
		end
		self.ui.layout:new_row()
	end

	self.ui:background(theme.background)
	self.ui.layout:col(c1 + c2 + c3 * 0.5)
	self.ui:label("Currently active: " .. project.settings.tuning.name)
	self.ui.layout:col(c3 * 0.5)
	if self.apply:update(self.ui) then
		-- print("HI")
		tuning.set(current_def)
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
