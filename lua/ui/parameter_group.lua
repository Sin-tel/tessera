local ui = require("ui/ui")
local ParameterGroup = {}

function ParameterGroup:new(name, paramlist)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.name = name
	new.collapse = false
	new.widgets = {}

	new.y = 0
	new.w = 0

	-- TODO: remove
	-- for _, v in ipairs(paramlist) do
	-- 	table.insert(new.widgets, v:intoWidget())
	-- end

	return new
end

function ParameterGroup:update(layout, ...)
	local x, y, w, h = ...
	self.y = y
	self.w = w

	if not self.collapse then
		for i, v in ipairs(self.widgets) do
			v:update(layout:row())
		end
	end
end

function ParameterGroup:draw(selected)
	local s = 32
	love.graphics.setColor(theme.ui_text)
	if self.collapse then
		util.drawText("+", 0, self.y, s, ui.ROW_HEIGHT, "left")
	else
		util.drawText("-", 0, self.y, s, ui.ROW_HEIGHT, "left")
	end
	util.drawText("    " .. self.name, 0, self.y, self.w - s, ui.ROW_HEIGHT, "left")

	if not self.collapse then
		love.graphics.setColor(theme.bg_nested)
		love.graphics.rectangle("fill", 0, self.y + ui.ROW_HEIGHT, self.w, #self.widgets * ui.ROW_HEIGHT)

		for i, v in ipairs(self.widgets) do
			local mode = false
			if v == selected then
				if self.action == "slider" then
					-- TODO: this doesn't even work
					mode = "press"
				else
					mode = "hover"
				end
			end
			v:draw(mode)
		end
	end
end

function ParameterGroup:getLength()
	if self.collapse then
		return 1
	end
	return #self.widgets + 1
end

function ParameterGroup.makeParameterGroups(channel)
	channel.parameterGroups = {}
	table.insert(channel.parameterGroups, ParameterGroup:new(channel.instrument.name, channel.instrument.parameters))
end

function ParameterGroup.addParameters(channel, effect)
	table.insert(channel.parameterGroups, ParameterGroup:new(effect.name, effect.parameters))
end

return ParameterGroup
