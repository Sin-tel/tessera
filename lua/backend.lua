-- @todo fix path when in subfolder
if release then
	package.cpath = package.cpath .. ";../target/release/?.dll"
else
	package.cpath = package.cpath .. ";../target/debug/?.dll"
end

local backend = require("rust_backend").init()

return backend
