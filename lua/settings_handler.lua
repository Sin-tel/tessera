local writefile = require("lib/serialize")

local function file_exists(name)
   local f=io.open(name,"r")
   if f~=nil then io.close(f) return true else return false end
end

local M = {}

function M.load()
   local settings = nil
   if file_exists("./settings.lua") then
      settings = require("./settings")
   else
      settings = {}
      settings.audio = {}
      settings.audio.default_host = "default"
      settings.audio.default_device = "default"
      settings.midi = {}
      settings.midi.default_input = "default"
      
      M.save(settings)
   end

   return settings
end

function M.save(settings)
   writefile("./settings", settings, "settings")
end




return M