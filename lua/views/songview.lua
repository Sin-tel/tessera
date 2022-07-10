songView = View:derive("Song")

function songView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	love.graphics.setColor(theme.ui_text)
	love.graphics.setFont(fonts.notes)

	drawText("THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG", 50, 50, w, h)
	drawText("the quick brown fox jumps over the lazy dog", 50, 70, w, h)
end
