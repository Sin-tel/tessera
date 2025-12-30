local Ui = require("ui.ui")
local widgets = require("ui.widgets")

local Device = {}
Device.__index = Device

-- options is a reference to an entry in device_list
function Device.new(data, options, meter_id)
	local self = setmetatable({}, Device)

	self.number = options.number

	-- reference to project data
	self.data = data
	self.state = data.state

	self.meter_id = meter_id
	self.meter_l = 0.0
	self.meter_r = 0.0

	-- copy of state that is already sent to backend
	self.state_old = {}
	self.mute_old = false

	-- UI stuff and parameter handlers
	self.collapse = widgets.CollapseDevice.new(self)
	self.elements = {}

	local index = 1

	for _, v in ipairs(options.parameters) do
		local w_name = v[1]
		local w_type = v[2] or w_name
		local w_options = v[3] or {}
		assert(w_name)
		local element = {}
		element.label = w_name
		element.name = w_name
		if w_type == "label" then
			element.widget = "label"
		elseif w_type == "separator" then
			element.widget = "separator"
		else
			if w_type == "slider" then
				local slider = widgets.Slider.new(self.state, index, w_options)
				if self.state[index] == nil then
					self.state[index] = slider.value.default
				end
				element.widget = slider
			elseif w_type == "selector" then
				local default = w_options.default or 1

				element.widget = widgets.Selector.new(self.state, index, w_options)
				if self.state[index] == nil then
					self.state[index] = default
				end
			elseif w_type == "dropdown" then
				local default = w_options.default or 1

				element.widget = widgets.Dropdown.new(self.state, index, w_options)
				if self.state[index] == nil then
					self.state[index] = default
				end
			elseif w_type == "toggle" then
				local default = w_options.default or false
				element.widget =
					widgets.Toggle.new(self.state, index, { label = w_name, style = "checkbox", default = default })
				element.label = nil

				if self.state[index] == nil then
					self.state[index] = default
				end
			else
				error(w_type .. " not supported!")
			end
			index = index + 1
		end
		table.insert(self.elements, element)
	end

	self.n_parameters = index - 1

	return self
end

function Device:update(ui, index, w)
	local start_x, start_y = ui.layout.start_x, ui.layout.y
	local w_label = util.clamp(w * 0.4 - 64, 0, Ui.PARAMETER_LABEL_WIDTH)
	w = w - Ui.PARAMETER_PAD * 2

	ui:background()
	if selection.device_index == index then
		ui:background(theme.bg_focus)
	end
	if self.collapse:update(ui) then
		ui:background(theme.bg_nested)
		ui:separator()

		for _, v in ipairs(self.elements) do
			if v.widget == "label" then
				ui.layout:col(w_label)
				ui.layout:col(w - w_label)

				ui:label(v.label, tessera.graphics.ALIGN_LEFT)
				ui.layout:new_row()
			elseif v.widget == "separator" then
				ui:separator()
			else
				ui.layout:col(w_label)
				if v.label then
					ui:label(v.label, tessera.graphics.ALIGN_RIGHT)
				end
				ui.layout:col(w - w_label)
				v.widget:update(ui)
				ui.layout:new_row()
			end
		end
		ui:separator()
	end

	-- detect hit anywhere inside of the device
	local end_y = ui.layout.y
	return ui:hit_area(start_x, start_y, w, end_y - start_y) and mouse.button_released
end

function Device:reset()
	self.state_old = {}
	self.mute_old = false
end

return Device
