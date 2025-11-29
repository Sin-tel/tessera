local resources = {}

-- fonts.main = tessera.graphics.newFont(12)
resources.fonts = {}
resources.fonts.main = tessera.graphics.new_font("Inter", 14)
resources.fonts.notes = tessera.graphics.new_font("Notes", 14)

tessera.graphics.set_font(resources.fonts.main)

resources.icons = {}
resources.icons.solo = tessera.graphics.new_image("assets/solo.png")
resources.icons.mute = tessera.graphics.new_image("assets/mute.png")
resources.icons.armed = tessera.graphics.new_image("assets/armed.png")
resources.icons.visible = tessera.graphics.new_image("assets/visible.png")
resources.icons.invisible = tessera.graphics.new_image("assets/invisible.png")
resources.icons.lock = tessera.graphics.new_image("assets/lock.png")
resources.icons.unlock = tessera.graphics.new_image("assets/unlock.png")

return resources
