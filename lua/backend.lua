-- kind of hacky, should just copy the dll into ./lib when building
local src = love.filesystem.getSource()
if release then
	package.cpath = package.cpath .. ";" .. src .. "/../target/release/?.dll"
else
	package.cpath = package.cpath .. ";" .. src .. "/../target/debug/?.dll"
end

local backend = require("tessera").init()

backend:setWorkingDirectory(love.filesystem.getSource())

return backend
