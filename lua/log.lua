io.stdout:setvbuf("no")

local log = {}

local file = io.open("out/out.log", "a")
assert(file, "Failed to create log file")
file:setvbuf("line")

log.messages = {}

function log.format(level, message)
	return string.format("[%s] %s", level, message)
end

function log.write(message)
	print(message)
	file:write(message)
	file:write("\n")
	file:flush()
	log.append(message)
end

function log.append(message)
	table.insert(log.messages, message)
end

function log.info(message)
	log.write(log.format("INFO", message))
end

function log.warn(message)
	log.write(log.format("WARN", message))
end

function log.error(message)
	log.write(log.format("ERROR", message))
end

return log
