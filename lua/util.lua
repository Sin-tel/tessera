function clamp(x, min, max)
  return x < min and min or (x > max and max or x)
end

function dist(x1,y1,x2,y2)
	return math.sqrt((x1-x2)^2 + (y1-y2)^2)
end

function length(x, y)
	return math.sqrt(x^2 + y^2)
end

function from_dB(x)
	return 10.0^(x/20.0)
end

function to_dB(x)
	return 20.0 * math.log10(x)
end

function deepcopy(orig, copies)
	copies = copies or {}
	local orig_type = type(orig)
	local copy
	if orig_type == 'table' then
		if copies[orig] then
			copy = copies[orig]
		else
			copy = {}
			copies[orig] = copy
			for orig_key, orig_value in next, orig, nil do
				copy[deepcopy(orig_key, copies)] = deepcopy(orig_value, copies)
			end
			setmetatable(copy, deepcopy(getmetatable(orig), copies))
		end
	else -- number, string, boolean, etc
		copy = orig
	end
	return copy
end

function drawText(str, x, y, w, h, align)
	align = align or "left"

	local f = love.graphics.getFont()
	local str2 = str
	local fh = f:getHeight()
	local fo = 0.5*(HEADER - fh)
	local l = str:len()
	local strip = false
	while true do
		local fw = f:getWidth(str2)
		if fw+2*fo < w then
			if align == "left" or strip then
				love.graphics.print(str2, math.floor(x + fo), math.floor(y + fo))
			elseif align == "center" then
				love.graphics.print(str2, math.floor(x + (w-fw)/2), math.floor(y + fo))
			elseif align == "right" then
				love.graphics.print(str2, math.floor(x + w-fw-fo), math.floor(y + fo))
			end
			break
		end
		l = l - 1
		if l <= 0 then
			break
		end
		str2 = str:sub(1,l) .. "..."
		strip = true
	end
end
