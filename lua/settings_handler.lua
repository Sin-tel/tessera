local util = require("util")
local writefile = require("lib/serialize")

local M = {}

M.path = love.filesystem.getSource()

function M.load()
   local setup
   if util.fileExists(M.path .. "/settings/setup.lua") then
      setup = require("settings/setup")
   else
      setup = {}
      setup.audio = {}
      setup.audio.default_host = "default"
      setup.audio.default_device = "default"
      setup.audio.buffer_size = 128
      setup.midi = {}
      setup.midi.inputs = { { name = "default" } }

      M.save(setup)
   end

   return setup
end

function M.save(setup)
   writefile(M.path .. "/settings/setup.lua", setup, "setup")
end

return M
