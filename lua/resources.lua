local resources = {}

-- fonts.main = love.graphics.newFont(12)
resources.fonts = {}
resources.fonts.main = love.graphics.newFont("Inter", 14)
resources.fonts.notes = love.graphics.newFont("Notes", 14)

love.graphics.setFont(resources.fonts.main)

resources.icons = {}
resources.icons.solo = love.graphics.newImage("res/solo.png")
resources.icons.mute = love.graphics.newImage("res/mute.png")
resources.icons.armed = love.graphics.newImage("res/armed.png")
resources.icons.visible = love.graphics.newImage("res/visible.png")
resources.icons.invisible = love.graphics.newImage("res/invisible.png")
resources.icons.lock = love.graphics.newImage("res/lock.png")
resources.icons.unlock = love.graphics.newImage("res/unlock.png")

return resources
