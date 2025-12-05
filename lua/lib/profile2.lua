local profile = require("jit.profile")
local prof = {}

-- see https://luajit.org/ext_profiler.html#ll_lua_api
prof.l1_stack_fmt = "plZ:"
prof.l2_stack_fmt = "F"
prof.profiler_fmt = "Fi1"

prof.running = false
prof.counts = {
    total = 0,
    vm_global = {},
    funcs = {},
}

local map_vmmode = {
    N = "Compiled",
    I = "Interpreted",
    C = "C code",
    G = "Garbage Collector",
    J = "JIT Compiler",
}

local function prof_cb(thread, samples, vmmode)
    local c = prof.counts

    c.total = c.total + samples
    c.vm_global[vmmode] = (c.vm_global[vmmode] or 0) + samples

    local l1 = profile.dumpstack(thread, prof.l1_stack_fmt, 1)
    local fname = profile.dumpstack(thread, "f", 1)

    if not l1 or l1 == "" then
        l1 = "(?)"
    end
    if string.find(fname, ":") then
        fname = "(?)"
    end

    local func_entry = c.funcs[l1]
    if not func_entry then
        func_entry = { total = 0, modes = {}, fname = fname }
        c.funcs[l1] = func_entry
    end
    func_entry.total = func_entry.total + samples
    func_entry.modes[vmmode] = (func_entry.modes[vmmode] or 0) + samples
end

function prof.start()
    if prof.running then
        return
    end

    profile.start(prof.profiler_fmt, prof_cb)
    prof.running = true
end

function prof.stop()
    if not prof.running then
        return
    end
    profile.stop()
    prof.running = false
end

function prof.report(limit)
    limit = limit or 20
    local c = prof.counts
    local total = c.total
    if total == 0 then
        print("No samples collected.")
        return
    end

    print(string.rep("-", 80))
    print(string.format("PROFILER REPORT (Total Samples: %d)", total))
    print(string.rep("-", 80))

    print("Global VM State:")
    for k, v in pairs(map_vmmode) do
        local count = c.vm_global[k] or 0
        local pct = (count / total) * 100
        if pct > 0.1 then
            print(string.format("  [%s] %-18s : %5.1f%%", k, v, pct))
        end
    end
    print("")

    local sorted_funcs = {}
    for name, data in pairs(c.funcs) do
        table.insert(sorted_funcs, { name = name, data = data })
    end
    table.sort(sorted_funcs, function(a, b)
        return a.data.total > b.data.total
    end)

    print(string.format("%-30s | %-20s | %6s | %s", "Source:Line", "Name", "Total%", "Mode Breakdown"))
    print(string.rep("-", 80))

    for i = 1, math.min(limit, #sorted_funcs) do
        local entry = sorted_funcs[i]
        local name = entry.name
        local d = entry.data
        local pct = (d.total / total) * 100

        local breakdown = ""
        for k, _ in pairs(map_vmmode) do
            local mode_count = d.modes[k] or 0
            if mode_count > 0 then
                local mode_pct = (mode_count / d.total) * 100
                if mode_pct > 5 then -- Only show significant modes
                    breakdown = breakdown .. string.format("%s:%d%% ", k, mode_pct)
                end
            end
        end

        print(string.format("%-30s | %-20s | %5.1f%% | %s", name, d.fname, pct, breakdown))
    end

    print(string.rep("-", 80))
end

return prof
