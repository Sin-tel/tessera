exclude_files = { "**/lib/*.lua" }

std = "max+tessera"
stds.tessera = {
   globals = {
      "tessera",
      "util",
      -- tables / modules
      "theme",
      "setup",
      "workspace",
      "mouse",
      "modifier_keys",
      "command",
      -- runtime stuff
      "audio_status",
      "selection",
      "clipboard",
      -- variables
      "release",
      "width",
      "height",
      "project",
      "ui_channels",
      "VERSION",
   },
   read_globals = {},
}

ignore = {
   "212", -- unused function arg
   "213", -- unused loop variable
   "561", -- cyclomatic complexity
}

-- allow_defined_top = true
