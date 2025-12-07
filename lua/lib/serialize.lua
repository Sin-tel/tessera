--[[
table serializer that outputs human readable lua code

supports nested tables (and arrays) of numbers, strings and booleans
doesn't support functions, userdata, circular references, and probably a lot of edge cases
]]

local infinity = math.huge
local writer = {}

local function get_writer(value)
	return writer[type(value)]
end

local function write_nan(n)
	return tostring(n) == tostring(0 / 0) and "0/0" or "-(0/0)"
end

-- serialize numbers
function writer.number(value)
	return value == infinity and "1/0"
		or value == -infinity and "-1/0"
		or value ~= value and write_nan(value)
		-- Note: we only care about preserving f32 representation
		or ("%.8G"):format(value)
end

-- serialize strings
function writer.string(value)
	return ("%q"):format(value):gsub("\\\n", "\\n")
end

-- serialize booleans
writer.boolean = tostring

local function is_array(t)
	local i = 0
	if type(t) ~= "table" then
		return false
	end
	for _ in pairs(t) do
		i = i + 1
		if t[i] == nil then
			return false
		end
	end
	return true
end

local function indent(b, n)
	table.insert(b, ("\t"):rep(n))
end

local function write_table(t, depth)
	local b = {}

	local arr = is_array(t)

	depth = depth + 1
	for k, v in pairs(t) do
		assert(type(k) ~= "function")
		assert(type(v) ~= "function")
		if type(v) == "table" then
			if type(k) == "string" then
				indent(b, depth)
				local s = ("%s"):format(k) .. " = {\n" .. write_table(v, depth) .. ",\n"
				table.insert(b, s)
			else
				if arr then
					indent(b, depth)
					local s = "{\n" .. write_table(v, depth) .. ",\n"
					table.insert(b, s)
				else
					indent(b, depth)
					local s = ("[%s]"):format(k) .. " = {\n" .. write_table(v, depth) .. ",\n"
					table.insert(b, s)
				end
			end
		else
			if type(k) == "string" then
				local write_value = get_writer(v)
				local value = write_value(v)

				indent(b, depth)
				local s = ("%s = %s,\n"):format(k, value)
				table.insert(b, s)
			else
				local write_key, write_value = get_writer(k), get_writer(v)
				local key, value = write_key(k), write_value(v)

				indent(b, depth)
				if arr then
					local s = ("%s,\n"):format(value)
					table.insert(b, s)
				else
					local s = ("[%s] = %s,\n"):format(key, value)
					table.insert(b, s)
				end
			end
		end
	end
	depth = depth - 1

	indent(b, depth)

	table.insert(b, "}")

	return table.concat(b)
end

local function serialize(t, var)
	if var then
		local s = "local " .. var .. " = {\n"
		s = s .. write_table(t, 0)
		s = s .. "\nreturn " .. var
		return s
	else
		return "{\n" .. write_table(t, 0)
	end
end

return serialize
