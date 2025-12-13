local tuning = require("tuning")

local pen = {}

pen.ox = 0
pen.oy = 0

function pen:mousepressed(canvas)
	local mx, my = canvas:get_mouse()

	self.ox = mx
	self.oy = my

	self.active = true
end

function pen:mousedown(canvas)
	--
end

function pen:mousereleased(canvas)
	self.active = false
end

function pen:draw(canvas)
	local mx, my = canvas:get_mouse()

	local f = canvas.transform:pitch_inv(my)

	-- TODO: query local grid
	local chromatic = math.floor(f + 0.5)
	local note = tuning.from_midi(chromatic)

	local p = tuning.get_pitch(note)
	local y = canvas.transform:pitch(p)

	local lx, ly = mx, my
	if self.active then
		lx, ly = self.ox, y
	end
	tessera.graphics.set_color(theme.text_tip)
	local note_name = tuning.get_name(note)
	tessera.graphics.text(note_name, lx + 5, ly - 20)

	if self.active then
		tessera.graphics.set_color(theme.white)
		tessera.graphics.line(self.ox, y, mx, y)
	end
end

return pen
