local resources = {}

-- fonts.main = tessera.graphics.newFont(12)
resources.fonts = {}
resources.fonts.main = tessera.graphics.newFont("Inter", 14)
resources.fonts.notes = tessera.graphics.newFont("Notes", 14)

tessera.graphics.setFont(resources.fonts.main)

resources.icons = {}
resources.icons.solo = tessera.graphics.newImage("assets/solo.png")
resources.icons.mute = tessera.graphics.newImage("assets/mute.png")
resources.icons.armed = tessera.graphics.newImage("assets/armed.png")
resources.icons.visible = tessera.graphics.newImage("assets/visible.png")
resources.icons.invisible = tessera.graphics.newImage("assets/invisible.png")
resources.icons.lock = tessera.graphics.newImage("assets/lock.png")
resources.icons.unlock = tessera.graphics.newImage("assets/unlock.png")

return resources
