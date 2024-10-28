--[[
table serializer that outputs human readable lua code

supports nested tables (and arrays) of numbers, strings and booleans 
doesn't support functions, userdata, circular references, and probably a lot of edge cases

-- TODO: use fast string concat
]]

local infinity = math.huge
local writer = {}

local function getWriter(value)
	return writer[type(value)]
end

local function writeNan(n)
	return tostring(n) == tostring(0 / 0) and "0/0" or "-(0/0)"
end

-- serialize numbers
function writer.number(value)
	return value == infinity and "1/0"
		or value == -infinity and "-1/0"
		or value ~= value and writeNan(value)
		or ("%.17G"):format(value)
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

local function writeTable(t, depth)
	local depth = depth or 0
	local s = ""

	local arr = is_array(t)

	depth = depth + 1
	for k, v in pairs(t) do
		if type(v) == "table" then
			if type(k) == "string" then
				s = s .. ("\t"):rep(depth) .. ("%s"):format(k) .. " = {\n" .. writeTable(v, depth) .. ",\n"
			else
				if arr then
					s = s .. ("\t"):rep(depth) .. "{\n" .. writeTable(v, depth) .. ",\n"
				else
					s = s .. ("\t"):rep(depth) .. ("[%s]"):format(k) .. " = {\n" .. writeTable(v, depth) .. ",\n"
				end
			end
		else
			if type(k) == "string" then
				local writeValue = getWriter(v)
				local value = writeValue(v)
				s = s .. ("\t"):rep(depth) .. ("%s = %s,\n"):format(k, value)
			else
				local writeKey, writeValue = getWriter(k), getWriter(v)
				local key, value = writeKey(k), writeValue(v)
				if arr then
					--[[if type(v) == "number" then
						if k == 1 then
							s = s .. ("\t"):rep(depth)
						end
						s = s ..  ('%s, '):format(value)
					else]]
					s = s .. ("\t"):rep(depth) .. ("%s,\n"):format(value)
					--end
				else
					s = s .. ("\t"):rep(depth) .. ("[%s] = %s,\n"):format(key, value)
				end
			end
		end
	end
	depth = depth - 1
	s = s .. ("\t"):rep(depth) .. "}"

	return s
end

local function writeTable2(t, var)
	local s = ""

	s = s .. (var .. " = {}\n")
	for k, v in pairs(t) do
		if type(v) == "table" then
			if type(k) == "string" then
				s = s .. writeTable2(v, ("%s.%s"):format(var, k))
			else
				s = s .. writeTable2(v, ("%s[%s]"):format(var, k))
			end
		else
			if type(k) == "string" then
				local writeValue = getWriter(v)
				local value = writeValue(v)
				s = s .. ("%s.%s = %s\n"):format(var, k, value)
			else
				local writeKey, writeValue = getWriter(k), getWriter(v)
				local key, value = writeKey(k), writeValue(v)
				s = s .. ("%s[%s] = %s\n"):format(var, key, value)
			end
		end
	end
	--end
	return s
end

local function serialize(t, var)
	local var = var or "t"

	local s = "local " .. var .. " = {\n"
	s = s .. writeTable(t)
	s = s .. "\nreturn " .. var
	return s
end

local function deserialize(s)
	return setfenv(loadstring(s), {})()
end

return serialize
