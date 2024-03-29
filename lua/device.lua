local widgets = require("ui/widgets")
local Device = {}

-- options is a reference to an entry in device_list
function Device:new(name, options)
	local new = {}
	setmetatable(new, self)
	self.__index = self

	new.number = options.number
	new.name = name

	-- UI stuff and parameter handlers
	new.collapse = widgets.Collapse:new(new.name)
	new.parameters = {}
	for _, v in ipairs(options.parameters) do
		local p = {}
		p.name = v[1]
		local widget_type = v[2]
		local widget_options = v[3] or {}
		if widget_type == "slider" then
			p.widget = widgets.Slider:new(widget_options)
		elseif widget_type == "selector" then
			p.widget = widgets.Selector:new(widget_options)
		elseif widget_type == "toggle" then
			p.widget = widgets.Toggle:new(p.name, { style = "checkbox", default = widget_options.default })
			p.name = nil
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
	if ui:put(self.collapse) then
		ui:background(theme.bg_nested)
		for _, v in ipairs(self.parameters) do
			ui.layout:col(w_label)
			if v.name then
				ui:label(v.name, "right")
			end
			ui.layout:col(w - w_label) -- max
			ui:put(v.widget)
			ui.layout:newRow()
		end
		ui:background()
	end
	local end_x, end_y = ui.layout.x, ui.layout.y

	-- detect hit anywhere inside of the device
	return ui:hitArea(start_x, start_y, w, end_y - start_y) and mouse.button_released
end

return Device
