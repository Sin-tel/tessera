local View = require("view")
local Ui = require("ui/ui")
local util = require("util")

local parameterView = View:derive("Parameters")

function parameterView:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.ui = Ui:new(new)

	return new
end

function parameterView:update()
	local w, h = self:getDimensions()

	self.ui:startFrame()

	self.ui.layout:start()

	-- TODO: should calculate this in device instead
	local w_label = util.clamp(w * 0.4 - 64, 0, Ui.PARAMETER_LABEL_WIDTH)

	if selection.channel then
		selection.channel.instrument:updateUi(self.ui, w, w_label)
		for _, v in ipairs(selection.channel.effects) do
			v:updateUi(self.ui, w, w_label)
		end
	end
end

function parameterView:draw()
	self.ui:draw()
end

return parameterView
