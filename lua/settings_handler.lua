local writefile = require("lib/serialize")

local function file_exists(name)
   local f=io.open(name,"r")
   if f~=nil then io.close(f) return true else return false end
end

local M = {}

function M.load()
   local setup = nil
   if file_exists("settings/setup.lua") then
      setup = require("settings/setup")
   else
      setup = {}
      setup.audio = {}
      setup.audio.default_host = "default"
      setup.audio.default_device = "default"
      setup.midi = {}
      setup.midi.default_input = "default"
      
      M.save(setup)
   end


   return settings
end

function M.save(setup)
   writefile("settings/setup", setup, "setup")
end




return M