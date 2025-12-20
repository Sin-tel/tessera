io.stdout:setvbuf("no")

local log = {}

local file = io.open("out/out.log", "a")
assert(file, "Failed to create log file")
file:setvbuf("line")

log.messages = {}

log.last_message = nil

local counter = 0

local function format(level, message)
	return string.format("[%s] %s", level, message)
end

function log.write(level, message)
	log.append(level, message)

	local message_fmt = format(level, message)
	print(message_fmt)
	file:write(message_fmt)
	file:write("\n")
	file:flush()
end

function log.append(level, message)
	local message_fmt = format(level, message)
	table.insert(log.messages, message_fmt)

	if level == "WARN" or level == "ERROR" then
		log.last_message = message
	end
end

function log.info(message)
	log.write("INFO", message)
end

function log.warn(message)
	log.write("WARN", message)
end

function log.error(message)
	log.write("ERROR", message)
end

function log.update(dt)
	if log.last_message then
		counter = counter + dt
		if counter > 3.0 then
			log.last_message = nil
		end
	end
end

return log
