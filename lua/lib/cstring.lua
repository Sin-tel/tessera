local ffi = require("ffi")

ffi.cdef([[
typedef const char* string;
]])

function cstring(str)
	-- need to add 1 for null termination
	local c_str = ffi.new("char[?]", #str + 1)
	ffi.copy(c_str, str)
	return c_str
end

return cstring
