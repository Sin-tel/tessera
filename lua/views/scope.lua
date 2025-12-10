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

	self.selector = widgets.Selector.new({ list = { "scope", "spectrum" } })

	self.lines = {}
	for i = 1, 7 do
		table.insert(self.lines, 4 ^ (1 - i))
	end

	return self
end

function Scope:update()
	self.ui:start_frame()
	self.ui.layout:col(Ui.scale(200))
	self.selector:update(self.ui, self, "index")
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
end

return Scope
