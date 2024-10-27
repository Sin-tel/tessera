exclude_files = { "**/lib/*.lua" }

std = "max+justidaw+love"
stds.love = {
   globals = { "love" },
}
stds.justidaw = {
   globals = {
      "util",
      -- tables
      "resources",
      "theme",
      "settings",
      "workspace",
      "mouse",
      "keyboard",
      "channelHandler",
      -- runtime stuff
      "audio_status",
      "selection",
      -- variables
      "release",
      "width",
      "height",
      "time",
      "project",
      "project_ui",
   },
   read_globals = {},
}

ignore = {
   "212", -- unused function arg
   "213", -- unused loop variable
   "561", -- cyclomatic complexity
}

-- allow_defined_top = true
