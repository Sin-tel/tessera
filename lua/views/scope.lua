local Ui = require("ui/ui")
local View = require("view")
local widgets = require("ui/widgets")
local Scope = View.derive("Scope")
Scope.__index = Scope

function Scope.new(spectrum)
	local self = setmetatable({}, Scope)

	self.index = 1
	if spectrum then
		self.index = 2
	end

	self.ui = Ui.new(self)

	self.selector = widgets.Selector.new(self, "index", { list = { "scope", "spectrum" } })

	self.lines = {}
	for i = 1, 7 do
		table.insert(self.lines, 4 ^ (1 - i))
	end

	return self
end

function Scope:update()
	self.ui:start_frame()
	self.ui.layout:col(Ui.scale(200))
	self.selector:update(self.ui)
	self.ui:end_frame()
end

function Scope:draw()
	tessera.graphics.set_color(theme.ui_text)

	if self.index == 2 then
		tessera.graphics.draw_spectrum(self.w, self.h)
	else
		tessera.graphics.draw_scope(self.w, self.h)
	end
	self.ui:draw()
	if self.index == 2 then
		self:draw_frequency()
	end
end

-- Tooltip with the frequency under the mouse.
function Scope:draw_frequency()
	local mx, my = self:get_mouse()
	if not self:focus() or mx < 0 or my < 0 or mx > self.w or my > self.h then
		return
	end
	local hz = tessera.graphics.spectrum_frequency(mx, self.w)
	if not hz then
		return
	end

	local text = string.format("%d Hz", math.floor(hz + 0.5))
	local w = tessera.graphics.measure_width(text) + 2 * Ui.PAD
	local h = Ui.ROW_HEIGHT
	-- below right of the cursor, flipped to stay inside the view
	local x = mx + Ui.PAD * 2
	local y = my + Ui.PAD * 2
	if x + w > self.w then
		x = mx - w - Ui.PAD
	end
	if y + h > self.h then
		y = my - h - Ui.PAD
	end

	tessera.graphics.set_color(theme.bg_menu)
	tessera.graphics.rectangle("fill", x, y, w, h, Ui.CORNER_RADIUS)
	tessera.graphics.set_color(theme.ui_text)
	tessera.graphics.set_font("main")
	tessera.graphics.set_font_size()
	tessera.graphics.label(text, x, y, w, h, tessera.graphics.ALIGN_CENTER)
end

return Scope
