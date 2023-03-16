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
   },
   read_globals = {},
}

max_cyclomatic_complexity = 15
