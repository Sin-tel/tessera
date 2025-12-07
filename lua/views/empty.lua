local Ui = require("ui/ui")
local View = require("view")
local widgets = require("ui/widgets")

local Empty = View.derive("Empty")
Empty.__index = Empty

function Empty.new()
	local self = setmetatable({}, Empty)

	return self
end

return Empty
