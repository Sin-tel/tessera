local Ui = require("ui/ui")
local deviceList = require("device_list")
local widgets = require("ui/widgets")
local View = require("view")

local Channels = View:derive("Channels")

function Channels:new()
	local new = {}
	setmetatable(new, self)
	self.__index = self

	-- new.select = nil
	new.ui = Ui:new(new)

	new.intrument_list = {}
	for k, v in pairs(deviceList.instruments) do
		table.insert(new.intrument_list, k)
	end
	new.dropdown = widgets.Dropdown:new({ title = "add instrument", list = new.intrument_list })

	return new
end

function Channels:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.ui:startFrame()
	self.ui.layout:padding()
	self.ui.layout:row(w)
	local add_instrument_index = self.ui:put(self.dropdown)

	self.ui.layout:padding(0)

	if add_instrument_index then
		local intrument_name = self.intrument_list[add_instrument_index]
		local ch = channelHandler:add(intrument_name)
	end

	for _, v in ipairs(channelHandler.list) do
		self.ui:put(v.widget)
	end

	if self.box.focus then
		self.index = math.floor(my / Ui.ROW_HEIGHT) + 1
	end
end

function Channels:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	-- if mouse.button == 1 then
	-- 	for i, v in ipairs(channelHandler.list) do
	-- 		if i == self.index then
	-- 			if mx < w - Ui.BUTTON_SMALL * 5 then
	-- 				selection.channel = v
	-- 			else
	-- 				local ind = math.floor(((mx - w) / Ui.BUTTON_SMALL) + 6)
	-- 				if ind == 1 then
	-- 					channelHandler:mute(v, not v.mute)
	-- 					for _, ch in ipairs(channelHandler.list) do
	-- 						ch.solo = false
	-- 					end
	-- 				elseif ind == 2 then
	-- 					if v.solo then
	-- 						for _, ch in ipairs(channelHandler.list) do
	-- 							ch.solo = false
	-- 							channelHandler:mute(ch, false)
	-- 						end
	-- 					else
	-- 						for _, ch in ipairs(channelHandler.list) do
	-- 							ch.solo = false
	-- 							channelHandler:mute(ch, true)
	-- 						end
	-- 						v.solo = true
	-- 						channelHandler:mute(v, false)
	-- 					end
	-- 				elseif ind == 3 then
	-- 					if v.armed then
	-- 						v.armed = false
	-- 					else
	-- 						for _, ch in ipairs(channelHandler.list) do
	-- 							ch.armed = false
	-- 						end
	-- 						v.armed = true
	-- 					end
	-- 				elseif ind == 4 then
	-- 					v.visible = not v.visible
	-- 				elseif ind == 5 then
	-- 					v.lock = not v.lock
	-- 				end
	-- 			end
	-- 		end
	-- 	end
	-- end
end

function Channels:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.ui:draw()

	-- for i, v in ipairs(channelHandler.list) do
	-- 	-- local y = Ui.ROW_HEIGHT * (i - 1) - self.scroll
	-- 	local y = 0
	-- 	if self.index == i then
	-- 		love.graphics.setColor(theme.bg_highlight)
	-- 		love.graphics.rectangle("fill", 0, y, w, Ui.ROW_HEIGHT)
	-- 	end

	-- 	love.graphics.setColor(theme.ui_text)
	-- 	if selection.channel == v then
	-- 		love.graphics.setColor(theme.highlight)
	-- 	end

	-- 	util.drawText(v.name, 0, y, w - Ui.BUTTON_SMALL * 5, Ui.ROW_HEIGHT, "left", true)

	-- 	if v.mute then
	-- 		love.graphics.setColor(theme.mute)
	-- 	else
	-- 		love.graphics.setColor(theme.text_dim)
	-- 	end
	-- 	love.graphics.draw(resources.icons.mute, w - Ui.BUTTON_SMALL * 5, y + 4)

	-- 	if v.solo then
	-- 		love.graphics.setColor(theme.solo)
	-- 	else
	-- 		love.graphics.setColor(theme.text_dim)
	-- 	end
	-- 	love.graphics.draw(resources.icons.solo, w - Ui.BUTTON_SMALL * 4, y + 4)

	-- 	if v.armed then
	-- 		love.graphics.setColor(theme.recording)
	-- 	else
	-- 		love.graphics.setColor(theme.text_dim)
	-- 	end
	-- 	love.graphics.draw(resources.icons.armed, w - Ui.BUTTON_SMALL * 3, y + 4)

	-- 	if v.visible then
	-- 		love.graphics.setColor(theme.ui_text)
	-- 		love.graphics.draw(resources.icons.visible, w - Ui.BUTTON_SMALL * 2, y + 4)
	-- 	else
	-- 		love.graphics.setColor(theme.text_dim)
	-- 		love.graphics.draw(resources.icons.invisible, w - Ui.BUTTON_SMALL * 2, y + 4)
	-- 	end

	-- 	if v.lock then
	-- 		if v.visible then
	-- 			love.graphics.setColor(theme.ui_text)
	-- 		else
	-- 			love.graphics.setColor(theme.text_dim)
	-- 		end
	-- 		love.graphics.draw(resources.icons.lock, w - Ui.BUTTON_SMALL * 1, y + 4)
	-- 	else
	-- 		love.graphics.setColor(theme.text_dim)
	-- 		love.graphics.draw(resources.icons.unlock, w - Ui.BUTTON_SMALL * 1, y + 4)
	-- 	end
	-- end
end

return Channels
