local widgets = require("ui/widgets")
local Device = {}

-- options is a reference to an entry in device_list
function Device:new(name, options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.number = options.number
	new.name = name

	-- TODO: get this out of here
	new.state = {}

	-- UI stuff and parameter handlers
	new.collapse = widgets.Collapse:new(new.name)
	new.parameters = {}
	for _, v in ipairs(options.parameters) do
		local p = {}
		local key = v[1]
		p.key = key
		p.label = key
		local widget_type = v[2]
		local widget_options = v[3] or {}
		if widget_type == "slider" then
			p.widget = widgets.Slider:new(widget_options)
			new.state[key] = p.widget.value.default
		elseif widget_type == "selector" then
			p.widget = widgets.Selector:new(widget_options)
			new.state[key] = widget_options.default or 1
		elseif widget_type == "toggle" then
			p.widget = widgets.Toggle:new(key, { style = "checkbox", default = widget_options.default })
			new.state[key] = widget_options.default
			p.label = nil
		else
			error(widget_type .. " not supported!")
		end

		table.insert(new.parameters, p)
	end

	return new
end

function Device:updateUi(ui, w, w_label)
	local start_x, start_y = ui.layout.x, ui.layout.y

	ui:background()
	if selection.device == self then
		ui:background(theme.bg_focus)
	end
	if self.collapse:update(ui) then
		ui:background(theme.bg_nested)
		for _, v in ipairs(self.parameters) do
			ui.layout:col(w_label)
			if v.label then
				ui:label(v.label, "right")
			end
			ui.layout:col(w - w_label) -- max
			v.widget:update(ui, self.state, v.key)
			ui.layout:newRow()
		end
		ui:background()
	end
	local end_x, end_y = ui.layout.x, ui.layout.y

	-- detect hit anywhere inside of the device
	return ui:hitArea(start_x, start_y, w, end_y - start_y) and mouse.button_released
end

return Device
