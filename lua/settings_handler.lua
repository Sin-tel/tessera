local writefile = require("lib/serialize")

local function file_exists(name)
   local f = io.open(name, "r")
   if f ~= nil then
      io.close(f)
      return true
   else
      return false
   end
end

local M = {}

M.path = love.filesystem.getSource()

function M.load()
   local setup
   if file_exists(M.path .. "/settings/setup.lua") then
      setup = require("settings/setup")
   else
      setup = {}
      setup.audio = {}
      setup.audio.default_host = "default"
      setup.audio.default_device = "default"
      setup.midi = {}
      setup.midi.inputs = { { name = "default" } }

      M.save(setup)
   end

   return setup
end

function M.save(setup)
   writefile(M.path .. "/settings/setup", setup, "setup")
end

return M
