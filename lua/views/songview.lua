local View = require("view")

local songView = View:derive("Song")

function songView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	love.graphics.setColor(theme.ui_text)
	love.graphics.setFont(resources.fonts.notes)

	util.drawText("THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG", 50, 50, w, h)
	util.drawText("thequickbrownfoxjumpsoverthelazydog{[()]}!@#$&*0123456789.+-/", 50, 70, w, h)
	util.drawText("5/4  8/7  A4  C5  Dt  Be  Fy  Bev  Fj  Eed", 50, 90, w, h)
end

return songView
