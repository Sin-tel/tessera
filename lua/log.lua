io.stdout:setvbuf("no")

local log = {}

local file = io.open("../out/out.log", "a")
file:setvbuf("full")

local function write_log(level, ...)
	local level_str = "[" .. level:upper() .. "] "

	io.stdout:write(level_str)
	print(...)

	file:write(level_str)
	local vlist = { ... }
	for i, v in ipairs(vlist) do
		if i > 1 then
			file:write("\t")
		end
		file:write(v)
	end
	file:write("\n")
	file:flush()
end

function log.info(...)
	write_log("info", ...)
end

function log.warn(...)
	write_log("warn", ...)
end

function log.error(...)
	write_log("error", ...)
end

return log
