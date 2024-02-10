io.stdout:setvbuf("no")

local real_print = print
local file = io.open("../out/out.log", "w")
function print(...)
	real_print(...)
	file:write(...)
	file:write("\n")
end
