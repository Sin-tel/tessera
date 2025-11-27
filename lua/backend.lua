-- kind of hacky, should just copy the dll into ./lib when building
-- local src = love.filesystem.getSource()
-- print(src)
-- package.cpath = package.cpath .. ";" .. src .. "/lib/?.dll"

-- local backend = require("tessera").init()
-- local backend = require("tessera")

-- ffi = require("ffi")
-- local backend = ffi.load("tessera")

-- backend:setWorkingDirectory(love.filesystem.getSource())

-- return backend

-- make shim
local backend = {}

function backend:ok()
	return true
end

function backend:updateScope() end
function backend:setup() end
function backend:insertChannel() end
function backend:insertEffect() end
function backend:pop() end
function backend:pitch() end
function backend:pressure() end
function backend:noteOff() end
function backend:noteOn() end
function backend:sendParameter() end
function backend:quit() end
function backend:midiOpenConnection() end
function backend:midiPorts()
	return {}
end
function backend:getScope()
	local t = {}
	for i = 1, 64 do
		t[i] = 0
	end
	return t
end
function backend:getSpectrum()
	local t = {}
	for i = 1, 64 do
		t[i] = 0
	end
	return t
end

return backend
