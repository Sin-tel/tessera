local View = require("view")

local Empty = View.derive("Empty")
Empty.__index = Empty

function Empty.new()
	local self = setmetatable({}, Empty)

	return self
end

return Empty
