local Ui = require("ui/ui")

-- TODO: commands
local function do_mute(ch_index, mute)
	local ch = project.channels[ch_index]
	if mute then
		ch.solo = false
	end
	if ch.mute ~= mute then
		ch.mute = mute
	end
end

local function do_solo(ch_index)
	local ch = project.channels[ch_index]
	if ch.solo then
		for i, v in ipairs(project.channels) do
			v.solo = false
			do_mute(i, false)
		end
	else
		for i, v in ipairs(project.channels) do
			if i == ch_index then
				v.solo = true
				do_mute(i, false)
			else
				v.solo = false
				do_mute(i, true)
			end
		end
	end
end

local function do_armed(ch_index)
	local ch = project.channels[ch_index]
	if ch.armed then
		ch.armed = false
	else
		for _, v in ipairs(project.channels) do
			v.armed = false
		end
		ch.armed = true
	end
end

local Button = {}
Button.__index = Button

local Channel = {}
Channel.__index = Channel

function Channel.new()
	local self = setmetatable({}, Channel)

	self.button_mute = Button.new({ img_on = tessera.image.mute, color_on = theme.mute })
	self.button_solo = Button.new({ img_on = tessera.image.solo, color_on = theme.solo })
	self.button_armed = Button.new({ img_on = tessera.image.armed, color_on = theme.recording })
	self.button_visible = Button.new({ img_on = tessera.image.visible, img_off = tessera.image.invisible })
	self.button_lock = Button.new({ img_on = tessera.image.lock, img_off = tessera.image.unlock })

	return self
end

function Channel:update(ui, ch_index)
	local x, y, w, h = ui:next()
	local p = Ui.PAD
	local b = Ui.BUTTON_SMALL

	ui:hitbox(self, x, y, w - 5 * b, h)

	local ch = project.channels[ch_index]

	if self.button_mute:update(ui, ch.mute, w - 5 * b, y + p, b, b) then
		do_mute(ch_index, not ch.mute)
	end
	if self.button_solo:update(ui, ch.solo, w - 4 * b, y + p, b, b) then
		do_solo(ch_index)
	end
	if self.button_armed:update(ui, ch.armed, w - 3 * b, y + p, b, b) then
		do_armed(ch_index)
	end
	if self.button_visible:update(ui, ch.visible, w - 2 * b, y + p, b, b) then
		ch.visible = not ch.visible

		if not ch.visible then
			selection.remove_channel(ch)
		end
	end
	if self.button_lock:update(ui, ch.lock, w - b, y + p, b, b) then
		ch.lock = not ch.lock

		if ch.lock then
			selection.remove_channel(ch)
		end
	end
	ui:push_draw(self.draw, { self, ui, ch_index, x, y, w, h })

	if ui.clicked == self then
		selection.ch_index = ch_index
		selection.device_index = nil
	end

	return ui.clicked == self
end

function Channel:draw(ui, ch_index, x, y, w, h)
	-- background
	local color_fill = nil
	if ui.hover == self then
		color_fill = theme.bg_highlight
	end

	if selection.ch_index == ch_index and selection.device_index == nil then
		color_fill = theme.bg_focus
	end

	if color_fill then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x, y, w, h)
	end

	local ch = project.channels[ch_index]

	-- label
	local c
	if selection.ch_index == ch_index then
		c = tessera.graphics.get_color_hsv(ch.hue / 360, 0.40, 0.95)
	else
		c = tessera.graphics.get_color_hsv(ch.hue / 360, 0.70, 0.90)
	end
	tessera.graphics.set_color(c)

	local w_buttons = 5 * Ui.BUTTON_SMALL

	local pad = 10
	local w_text = w - w_buttons - 2 * pad
	tessera.graphics.label(project.channels[ch_index].name, x + pad, y, w_text * 0.6 - 16, h)

	-- meter
	ch = ui_channels[ch_index]

	local device = ch.instrument
	if #ch.effects > 0 then
		device = ch.effects[#ch.effects]
	end
	local ml = device.meter_l
	local mr = device.meter_r

	local cl = util.meter_color(ml)
	local cr = util.meter_color(mr)

	local wl = util.clamp((util.to_dB(ml) + 80) / 80, 0, 1)
	local wr = util.clamp((util.to_dB(mr) + 80) / 80, 0, 1)

	local x1 = w_text * 0.6
	local h1 = Ui.scale(16)
	local y1 = y + 0.5 * (h - h1)
	local w1 = w - x1 - w_buttons - 20
	local h2 = 0.5 * h1
	if w1 > 0 then
		tessera.graphics.set_color(theme.bg_nested)
		tessera.graphics.rectangle("fill", x1, y1, w1, h1)
		if wl > 0 then
			tessera.graphics.set_color(cl)
			tessera.graphics.rectangle("fill", x1, y1, w1 * wl, h2 - 1)
		end
		if wr > 0 then
			tessera.graphics.set_color(cr)
			tessera.graphics.rectangle("fill", x1, y1 + h2, w1 * wr, h2)
		end
		tessera.graphics.set_color(theme.line)
		tessera.graphics.rectangle("line", x1, y1 - 0.5, w1, h1, 2)

		-- tessera.graphics.set_color(theme.line)
		-- tessera.graphics.rectangle("line", x1, y1, w1, h1, 2)
	end
end

function Button.new(options)
	local self = setmetatable({}, Button)

	self.img_on = options.img_on
	self.img_off = options.img_off or options.img_on
	self.color_on = options.color_on or theme.ui_text
	self.color_off = theme.text_dim

	return self
end

function Button:update(ui, checked, x, y, w, h)
	ui:hitbox(self, x, y, w, h)
	ui:push_draw(self.draw, { self, ui, checked, x, y, w, h })

	return ui.clicked == self
end

function Button:draw(ui, checked, x, y, w, h)
	local color_fill
	if ui.hover == self and ui.active ~= self then
		color_fill = theme.bg_highlight
	end
	if color_fill then
		tessera.graphics.set_color(color_fill)
		tessera.graphics.rectangle("fill", x - 1, y - 1, w, h, Ui.CORNER_RADIUS)
	end

	if checked then
		tessera.graphics.set_color(self.color_on)
		tessera.graphics.draw(self.img_on, x, y)
	else
		tessera.graphics.set_color(self.color_off)
		tessera.graphics.draw(self.img_off, x, y)
	end
end

return Channel
