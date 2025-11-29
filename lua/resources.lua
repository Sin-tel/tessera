local resources = {}

-- fonts.main = love.graphics.newFont(12)
resources.fonts = {}
resources.fonts.main = love.graphics.newFont("Inter", 14)
resources.fonts.notes = love.graphics.newFont("Notes", 14)

love.graphics.setFont(resources.fonts.main)

resources.icons = {}
resources.icons.solo = love.graphics.newImage("assets/solo.png")
resources.icons.mute = love.graphics.newImage("assets/mute.png")
resources.icons.armed = love.graphics.newImage("assets/armed.png")
resources.icons.visible = love.graphics.newImage("assets/visible.png")
resources.icons.invisible = love.graphics.newImage("assets/invisible.png")
resources.icons.lock = love.graphics.newImage("assets/lock.png")
resources.icons.unlock = love.graphics.newImage("assets/unlock.png")

return resources
