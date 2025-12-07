-- Minimal utf8 handline to calculate offsets and substrings

local utf8 = {}

-- Calculate the length
function utf8.len(s)
    local len = 0
    for i = 1, #s do
        local byte = s:byte(i)
        if byte < 128 or byte >= 192 then
            len = len + 1
        end
    end
    return len
end

-- Get character at position i
function utf8.char(s, i)
    return string.sub(s, i, utf8.next(s, i) - 1)
end

-- Get substring
function utf8.sub(s, i, j)
    assert(i <= j)
    local i2 = i
    for _ = 0, j - i do
        i2 = utf8.next(s, i2)
    end
    return string.sub(s, i, i2 - 1)
end

-- Get get byte offset of previous character
function utf8.prev(s, byte_pos)
    byte_pos = byte_pos - 1
    while byte_pos > 1 do
        local byte = s:byte(byte_pos)
        if byte < 128 or byte >= 192 then
            break
        end
        byte_pos = byte_pos - 1
    end
    return byte_pos
end

-- Get get byte offset of next character
function utf8.next(s, byte_pos)
    byte_pos = byte_pos + 1
    while byte_pos <= #s do
        local byte = s:byte(byte_pos)
        if byte < 128 or byte >= 192 then
            break
        end
        byte_pos = byte_pos + 1
    end
    return byte_pos
end

return utf8
