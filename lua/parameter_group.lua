local ui = require("ui")
local Slider = require("slider")

local ParameterGroup = {}

function ParameterGroup:new(name, paramlist)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.name = name
	new.collapse = false
	new.sliders = {}

	for _, v in ipairs(paramlist) do
		table.insert(new.sliders, Slider:new(v))
	end

	return new
end

function ParameterGroup:draw(y, w, selected)
	local y0 = ui.GRID * y

	local s = 32
	love.graphics.setColor(theme.ui_text)
	if self.collapse then
		util.drawText("+", 0, y0, s, ui.GRID, "left")
	else
		util.drawText("-", 0, y0, s, ui.GRID, "left")
	end
	util.drawText("    " .. self.name, 0, y0, w - s, ui.GRID, "left")

	if not self.collapse then
		love.graphics.setColor(theme.bg_nested)
		love.graphics.rectangle("fill", 0, y0 + ui.GRID, w, #self.sliders * ui.GRID)

		for i, v in ipairs(self.sliders) do
			local mode = false
			if v == selected then
				if self.action == "slider" then
					mode = "press"
				else
					mode = "hover"
				end
			end
			v:draw(y0 + i * ui.GRID, w, mode)
		end
	end
end

function ParameterGroup:getLength()
	if self.collapse then
		return 1
	end
	return #self.sliders + 1
end

function ParameterGroup.makeParameterGroups(channel)
	channel.parameterGroups = {}
	table.insert(channel.parameterGroups, ParameterGroup:new("channel", channel.parameters))
	table.insert(channel.parameterGroups, ParameterGroup:new(channel.instrument.name, channel.instrument.parameters))
end

function ParameterGroup.addParameters(channel, effect)
	table.insert(channel.parameterGroups, ParameterGroup:new(effect.name, effect.parameters))
end

return ParameterGroup
