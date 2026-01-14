local Roll = require("roll")
local Ui = require("ui.ui")
local log = require("log")
local tuning = require("tuning")
local widgets = require("ui.widgets")

local function channel_group()
	-- get all channels in a group (there is only a single group right now)
	local group = {}
	for i, v in ipairs(project.channels) do
		if i > 1 then
			table.insert(group, v)
		end
	end
	return group
end

-- TODO: fix master
local function solo_mute(ch)
	local group = channel_group()
	local all = true
	for _, v in ipairs(group) do
		all = all and v.mute
	end
	if all then
		for _, v in ipairs(group) do
			v.mute = false
		end
	else
		for _, v in ipairs(group) do
			v.mute = true
		end
		ch.mute = false
	end
end

local function solo_visible(ch)
	local group = channel_group()
	local all = false
	for _, v in ipairs(group) do
		all = all or v.visible
	end

	if all then
		for _, v in ipairs(group) do
			v.visible = false
		end
		ch.visible = true
	else
		for _, v in ipairs(group) do
			v.visible = true
		end
	end
end

local function solo_lock(ch)
	local group = channel_group()
	local all = true
	for _, v in ipairs(group) do
		all = all and v.lock
	end
	if all then
		for _, v in ipairs(group) do
			v.lock = false
		end
	else
		for _, v in ipairs(group) do
			v.lock = true
		end
		ch.lock = false
	end
end

local Channel = {}
Channel.__index = Channel

function Channel.new(ch_index, data, instrument, meter_id)
	local self = setmetatable({}, Channel)

	self.ch_index = ch_index

	-- reference to project data
	self.data = data
	self.mute_old = false
	self.gain_old = nil

	if instrument then
		self.instrument = instrument
		self.roll = Roll.new(ch_index)
	end
	self.effects = {}

	self.meter_id = meter_id
	self.meter_l = 0.0
	self.meter_r = 0.0

	-- UI widgets
	self.gain_slider = widgets.Slider.new(self.data, "gain", { default = 0, max = 12, t = "dB" })

	self.meter = widgets.Meter.new(self)

	self.button_mute = widgets.ToggleSmall.new({ label = "m", color_on = theme.mute })
	self.button_armed = widgets.ToggleSmall.new({ img_on = tessera.icon.armed, color_on = theme.recording })
	self.button_visible = widgets.ToggleSmall.new({ img_on = tessera.icon.visible, img_off = tessera.icon.invisible })

	-- TODO: this is confusing, should swap logic to use "unlocked"
	self.button_lock = widgets.ToggleSmall.new({
		img_on = tessera.icon.lock,
		img_off = tessera.icon.unlock,
		color_on = theme.text_dim,
		color_off = theme.ui_text,
	})

	return self
end

function Channel:update(ui, ch_index, bg_color, w)
	ui:background(bg_color)

	local start_x, start_y = ui.layout.start_x, ui.layout.y

	local w1 = w * 0.5
	local w2 = w - w1

	local w_button = Ui.ROW_HEIGHT
	local w_pad = w2 - w_button * 4 - Ui.PAD

	ui.layout:col(w1)
	local color
	local ch = project.channels[ch_index]
	if selection.ch_index == ch_index then
		color = tessera.graphics.get_color_hsv(ch.hue / 360, 0.40, 0.95)
	else
		color = tessera.graphics.get_color_hsv(ch.hue / 360, 0.70, 0.90)
	end
	ui:label(self.data.name, { align = tessera.graphics.ALIGN_LEFT, color = color })
	if w_pad > 0 then
		ui.layout:col(w_pad)
	end

	if ch_index > 1 then
		ui.layout:padding(Ui.scale(2))
		ui.layout:col(w_button)
		if self.button_armed:update(ui, ch.armed) then
			if ch.armed then
				ch.armed = false
			else
				if not modifier_keys.shift then
					for _, v in ipairs(project.channels) do
						v.armed = false
					end
				end
				ch.armed = true
			end
		end

		ui.layout:col(w_button)
		if self.button_mute:update(ui, ch.mute) then
			if ui.double_click then
				solo_mute(ch)
			else
				ch.mute = not ch.mute
			end
		end

		ui.layout:col(w_button)
		if self.button_visible:update(ui, ch.visible) then
			if ui.double_click then
				solo_visible(ch)
			else
				ch.visible = not ch.visible
			end
			selection.remove_inactive()
		end

		ui.layout:col(w_button)
		if self.button_lock:update(ui, ch.lock) then
			if ui.double_click then
				solo_lock(ch)
			else
				ch.lock = not ch.lock
			end
			selection.remove_inactive()
		end
	end

	ui.layout:padding()
	ui.layout:new_row()
	ui.layout:col(w1)
	self.gain_slider:update(ui)
	ui.layout:col(w2)
	self.meter:update(ui)

	ui.layout:new_row()

	local end_y = ui.layout.y
	ui:hitbox(self, start_x, start_y, w, end_y - start_y)

	return ui.clicked == self
end

function Channel:event(event)
	if self.instrument then
		self.roll:event(event)
		self:send_event(event)
	end
end

-- send an event to the backend
function Channel:send_event(event)
	local token = event.token
	if event.name == "note_on" then
		local pitch = tuning.get_pitch(event.interval)
		local v_curve = util.velocity_curve(event.vel)
		tessera.audio.note_on(self.ch_index, pitch, event.offset, v_curve, token)
	elseif event.name == "note_off" then
		tessera.audio.note_off(self.ch_index, token)
	elseif event.name == "pitch" then
		tessera.audio.pitch(self.ch_index, event.offset, token)
	elseif event.name == "pressure" then
		tessera.audio.pressure(self.ch_index, event.pressure, token)
	elseif event.name == "sustain" then
		tessera.audio.sustain(self.ch_index, event.sustain)
	else
		log.warn("unhandled event: ", util.dump(event))
	end
end

function Channel:reset()
	self.mute_old = false
	self.gain_old = nil
end

return Channel
