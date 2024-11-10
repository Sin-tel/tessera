local resources = {}

-- fonts.main = love.graphics.newFont(12)
resources.fonts = {}
resources.fonts.main = love.graphics.newFont("res/dejavu_normal.fnt", "res/dejavu_normal.png")
resources.fonts.notes = love.graphics.newImageFont(
	"res/font_notes.png",
	" ABCDEFGHIJKLMNOPQRSTUVWXYZ"
		.. "0123456789.+-/"
		.. "qwerty" -- flats/sharps  bbb, bb, b, n, #,  ##
		.. "asdfgh" -- plus/min      ---, --, -, +, ++, +++
		.. "zxcvbn" -- septimals     LLL, LL, L, 7, 77, 777
		.. "iopjkl" -- quartersharps ddd, dd, d, t, tt, ttt
		.. "{[()]}" -- ups/downs     vvv, vv, v, ^, ^^, ^^^
		.. "!@#$&*", -- arrows       vvv, vv, v, ^, ^^, ^^^
	-1
)

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
