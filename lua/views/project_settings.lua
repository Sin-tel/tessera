local Notation = require("notation")
local Ui = require("ui/ui")
local View = require("view")
local log = require("log")
local tuning = require("tuning")
local tuning_presets = require("default.tuning_presets")
local widgets = require("ui/widgets")

local ProjectSettings = View.derive("Project Settings")
ProjectSettings.__index = ProjectSettings

-- One choice of notation for the current tuning, and how it writes the primes.
local NotationRow = {}
NotationRow.__index = NotationRow

-- Compare two NotationChoice's, given they are for the same temperament.
local function same_choice(a, b)
	return a.accidentals == b.accidentals and a.half_sharp == b.half_sharp and a.johnston == b.johnston
end

-- option is a NotationOption, see tuning.rs
function NotationRow.new(view, option)
	local self = setmetatable({}, NotationRow)

	self.view = view
	self.option = option

	local notation = Notation.new(option.style)

	local n = option.choice.accidentals
	self.title = "Plain"
	if n == 1 then
		self.title = "Single"
	elseif n > 0 then
		self.title = tostring(n) .. " accidentals"
	end
	if option.choice.half_sharp then
		self.title = "Neutral"
	end
	if option.style.johnston then
		self.title = "Johnston"
	end

	-- how the prime is written on its nominal, and the alternative if there is one
	self.primes = {}
	for i, s in ipairs(option.spellings) do
		local written = {}
		if s.nominal then
			table.insert(written, notation:name(s.nominal))
		end
		if s.alternative then
			local name = notation:name(s.alternative)
			-- half sharps can write both spellings the same way
			if name ~= written[1] then
				table.insert(written, name)
			end
		end
		self.primes[i] = {
			ratio = s.ratio,
			name = table.concat(written, " = "),
		}
	end

	return self
end

-- The notation that is in use.
function NotationRow:is_current()
	return self.view.option ~= nil and same_choice(self.view.option.choice, self.option.choice)
end

function NotationRow:update(ui)
	local x, y, w, h = ui:next()
	ui:hitbox(self, x, y, w, h)
	ui:push_draw(self.draw, { self, ui, x, y, w, h })
	return ui.clicked == self
end

function NotationRow:draw(ui, x, y, w, h)
	local title_w = 0.32 * w
	if self:is_current() then
		tessera.graphics.set_color(theme.widget)
		tessera.graphics.rectangle("fill", x - 2 * Ui.PAD, y, title_w - 4 * Ui.PAD, h, Ui.CORNER_RADIUS)
	elseif ui.active == self then
		tessera.graphics.set_color(theme.widget_press)
		tessera.graphics.rectangle("fill", x - 2 * Ui.PAD, y, title_w - 4 * Ui.PAD, h, Ui.CORNER_RADIUS)
	elseif ui.hover == self and ui.active ~= self then
		tessera.graphics.set_color(theme.line_hover)
		tessera.graphics.rectangle("line", x - 2 * Ui.PAD, y, title_w - 4 * Ui.PAD, h, Ui.CORNER_RADIUS)
	end

	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.label(self.title, x, y, title_w, h, tessera.graphics.ALIGN_LEFT)

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

	local category_names = {}
	for i, c in ipairs(tuning_presets.categories) do
		category_names[i] = c.name
	end

	self.tuning_mode = 1
	self.select_tuning_mode = widgets.Selector.new(self, "tuning_mode", { list = category_names, no_undo = true })

	self.preset_index = 1
	self.preset_dropdown = widgets.Dropdown.new(self, "preset_index", { list = {}, arrows = true, no_undo = true })

	self.apply = widgets.Button.new("Apply")

	-- The project's definition we last synced with.
	self.active = nil
	-- Copy of a definition that is being edited, applied with the button.
	self.def = nil
	-- Selected notation option for self.def.
	self.option = nil
	self.rows = {}

	return self
end

-- Preset keys in the selected category.
function ProjectSettings:preset_keys()
	return tuning_presets.categories[self.tuning_mode].list
end

function ProjectSettings:set_category(index)
	self.tuning_mode = index
	local names = {}
	for i, k in ipairs(self:preset_keys()) do
		names[i] = tuning_presets.tunings[k].name
	end
	self.preset_dropdown.list = names
end

function ProjectSettings:select_notation(option)
	self.option = option
	self.def.notation = util.clone(option.choice)
end

-- Start editing a preset, with its recommended notation.
function ProjectSettings:select_preset(key)
	self.def = util.clone(tuning_presets.tunings[key])
	self.preset_dropdown.title = nil
	self:update_rows()
end

-- Show the active definition, if the project's changed since the last time.
function ProjectSettings:sync()
	local active = project.settings.tuning
	assert(active)
	if active == self.active then
		return
	end
	self.active = active
	self.def = util.clone(active)

	self:set_category(tuning_presets.category(active))

	-- Definitions that aren't a preset only show their name.
	self.preset_index = 1
	self.preset_dropdown.title = active.name or active.subgroup
	local key = tuning_presets.find(active)
	for i, k in ipairs(self:preset_keys()) do
		if k == key then
			self.preset_index = i
			self.preset_dropdown.title = nil
		end
	end

	self:update_rows()

	-- update_rows picked the recommended notation, prefer the one that's in use
	for _, row in ipairs(self.rows) do
		if same_choice(active.notation, row.option.choice) then
			self:select_notation(row.option)
		end
	end
end

-- Rebuild the notation options for self.def, and select the recommended one.
function ProjectSettings:update_rows()
	self.rows = {}
	self.option = nil
	self.error = nil

	local ok, options = pcall(tessera.tuning.notations, self.def)
	if not ok then
		log.error(options)
		self.error = options
		return
	end

	for _, option in ipairs(options) do
		table.insert(self.rows, NotationRow.new(self, option))
	end

	-- first recommended one, or else the first one
	for _, row in ipairs(self.rows) do
		if row.option.recommended then
			self:select_notation(row.option)
			return
		end
	end
	if self.rows[1] then
		self:select_notation(self.rows[1].option)
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
	self:sync()

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
		if self.tuning_mode == tuning_presets.category(self.active) then
			-- back to the active one
			self.active = nil
			self:sync()
		else
			self:set_category(self.tuning_mode)
			self.preset_index = 1
			self:select_preset(self:preset_keys()[1])
		end
	end

	self.ui.layout:new_row()
	self.ui.layout:col(c1)
	self.ui.layout:col(c2)
	self.ui:label("Tuning")
	self.ui.layout:col(c3)

	if self.preset_dropdown:update(self.ui) then
		self:select_preset(self:preset_keys()[self.preset_index])
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
			self:select_notation(row.option)
		end
		self.ui.layout:new_row()
	end

	self.ui:background(theme.background)
	self.ui.layout:col(c1 + c2)

	if self.apply:update(self.ui) and self.option then
		tuning.set(self.def)
	end
	self.ui.layout:new_row()

	self.ui:label("Currently active: " .. project.settings.tuning.name)

	-- draw chromatic midi input map
	-- TODO: show ET steps when applicable
	-- TODO: draw actual black/white keyboard for clarity (maybe highlight currently playing notes?)
	-- TODO: this interacts with current tuning center and relative_note_names, make sure ratio also does
	self.ui:background(theme.background)
	self.ui:label("Input map")
	self.ui.layout.h = Ui.scale(24)
	for _, key in ipairs(tuning.input_map()) do
		self.ui:background(key.black and theme.background or theme.bg_highlight)
		local color = theme.ui_text
		self.ui.layout:col(c1)
		self.ui.layout:col(c2)
		self.ui:label(key.name, { color = color, font = "notes" })
		self.ui.layout:col(c3)
		self.ui:label(key.ratio, { color = color, font = "notes" })
		self.ui.layout:new_row()
	end
	self.ui.layout.h = Ui.scale(32)

	self.ui:end_frame()
end

function ProjectSettings:draw()
	self.ui:draw()
end

function ProjectSettings:mousepressed()
	--
end

return ProjectSettings
