SongView =  View:derive("Song")

function SongView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	love.graphics.setColor(Theme.ui_text)
	love.graphics.setFont(font_notes)

	drawText("THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG", 50,50,w,h)
		drawText("the quick brown fox jumps over the lazy dog", 50,70,w,h)

end
