local Ui = require("ui/ui")
local util = require("util")

local Button = {}

local Channel = {}

function Channel:new(channel)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.channel = channel

	new.button_mute = Button:new({ img_on = resources.icons.mute, color_on = theme.mute })
	new.button_solo = Button:new({ img_on = resources.icons.solo, color_on = theme.solo })
	new.button_armed = Button:new({ img_on = resources.icons.armed, color_on = theme.recording })
	new.button_visible = Button:new({ img_on = resources.icons.visible, img_off = resources.icons.invisible })
	new.button_lock = Button:new({ img_on = resources.icons.lock, img_off = resources.icons.unlock })

	return new
end

function Channel:update(ui, x, y, w, h)
	local p = Ui.DEFAULT_PAD
	local b = Ui.BUTTON_SMALL

	ui:hitbox(self, x, y, w - 5 * b, h)

	local ch = self.channel

	if self.button_mute:update(ui, ch.mute, w - 5 * b, y + p, b, b) then
		channelHandler:mute(ch, not ch.mute)
	end
	if self.button_solo:update(ui, ch.solo, w - 4 * b, y + p, b, b) then
		channelHandler:solo(ch)
	end
	if self.button_armed:update(ui, ch.armed, w - 3 * b, y + p, b, b) then
		channelHandler:armed(ch)
	end
	if self.button_visible:update(ui, ch.visible, w - 2 * b, y + p, b, b) then
		ch.visible = not ch.visible
	end
	if self.button_lock:update(ui, ch.lock, w - b, y + p, b, b) then
		ch.lock = not ch.lock
	end
	ui:pushDraw(self.draw, self, ui, x, y, w, h)

	if ui.clicked == self then
		selection.channel = self.channel
		selection.device = nil
	end

	return ui.clicked == self
end

function Channel:draw(ui, x, y, w, h)
	local color_fill = nil
	if ui.hover == self then
		color_fill = theme.bg_highlight
	end

	if selection.channel == self.channel and selection.device == nil then
		color_fill = theme.bg_focus
	end

	if color_fill then
		love.graphics.setColor(color_fill)
		love.graphics.rectangle("fill", x, y, w, h)
	end

	love.graphics.setColor(theme.ui_text)
	if selection.channel == self.channel then
		love.graphics.setColor(theme.highlight)
	end

	util.drawText(self.channel.name, x, y, w, h, "left", true)
end

function Button:new(options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.img_on = options.img_on
	new.img_off = options.img_off or options.img_on
	new.color_on = options.color_on or theme.ui_text
	new.color_off = theme.text_dim

	return new
end

function Button:update(ui, checked, x, y, w, h)
	ui:hitbox(self, x, y, w, h)
	ui:pushDraw(self.draw, self, ui, checked, x, y, w, h)

	return ui.clicked == self
end

function Button:draw(ui, checked, x, y, w, h)
	local color_fill
	if ui.hover == self and ui.active ~= self then
		color_fill = theme.bg_highlight
	end
	if color_fill then
		love.graphics.setColor(color_fill)
		love.graphics.rectangle("fill", x - 1, y - 1, w, h, Ui.CORNER_RADIUS)
	end

	if checked then
		love.graphics.setColor(self.color_on)
		love.graphics.draw(self.img_on, x, y)
	else
		love.graphics.setColor(self.color_off)
		love.graphics.draw(self.img_off, x, y)
	end
end

return Channel
