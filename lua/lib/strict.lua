-- Make declaring globals in functions illegal

local mt = {}

setmetatable(_G, mt)

mt.__newindex = function(t, n, v)
    if debug.getinfo(2) then
        local w = debug.getinfo(2, "S").what
        local src = debug.getinfo(2, "S").source
        -- print(debug.getinfo(2, "S").source, n)

        -- if w ~= "main" and w ~= "C" or src ~= "@src\\main.lua" then
        if w ~= "main" then
            error("Script attempted to create global variable '" .. tostring(n) .. "'", 2)
        end
    end
    rawset(t, n, v)
end

mt.__index = function(t, n)
    if debug.getinfo(2) and debug.getinfo(2, "S").what ~= "C" then
        error("Script attempted to access unexisting global variable '" .. tostring(n) .. "'", 2)
    end
    return rawget(t, n)
end
