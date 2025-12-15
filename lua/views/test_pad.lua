local View = require("view")
local tuning = require("tuning")

local TestPadView = View.derive("note pad")
TestPadView.__index = TestPadView

function TestPadView.new()
	local self = setmetatable({}, TestPadView)

	self.v = 0
	self.f = 60
	self.chromatic = 60

	self.token = nil

	return self
end

function TestPadView:draw()
	local mx, my = self:get_mouse()

	local x1 = 4
	local y1 = 4
	local x2 = self.w - 4
	local y2 = self.h - 4

	tessera.graphics.set_color(theme.bg_nested)
	tessera.graphics.rectangle("fill", x1, y1, x2 - x1, y2 - y1)

	tessera.graphics.set_color(theme.line)
	tessera.graphics.rectangle("line", x1, y1, x2 - x1, y2 - y1)

	local oct = math.floor(self.w / 200)
	if oct < 1 then
		oct = 1
	end
	for i = 1, oct - 1 do
		local xx = x1 + i * (x2 - x1) / oct
		tessera.graphics.line(xx, y1, xx, y2)
	end

	tessera.graphics.set_color(theme.ui_text)

	mx = util.clamp(mx, x1, x2)
	my = util.clamp(my, y1, y2)

	if self.box.focus then
		tessera.graphics.circle("line", mx, my, 5)

		local mxx = (mx - x1) / (x2 - x1)
		local myy = (my - y1) / (y2 - y1)

		self.f = 60 - math.floor(oct * 0.5) * 12 + oct * 12 * mxx
		self.chromatic = math.floor(0.5 + self.f)
		self.v = 1.0 - myy

		local ch_index = selection.ch_index
		if (mouse.button == 1 or mouse.button == 2) and ch_index then
			ui_channels[ch_index]:event({
				name = "pitch",
				token = self.token,
				offset = self.f - self.pitch,
			})
			ui_channels[ch_index]:event({
				name = "pressure",
				token = self.token,
				pressure = self.v,
			})
		end
	end
end

function TestPadView:mousepressed()
	local ch_index = selection.ch_index
	if (mouse.button == 1 or mouse.button == 2) and ch_index then
		self.token = tessera.audio.get_token()
		local vel = self.v
		local note = tuning.from_midi(self.chromatic)
		self.pitch = tuning.get_pitch(note)

		ui_channels[ch_index]:event({
			name = "note_on",
			token = self.token,
			pitch = note,
			vel = vel,
			offset = self.f - self.pitch,
		})
	end
end

function TestPadView:mousereleased()
	local ch_index = selection.ch_index
	if mouse.button == 1 and ch_index then
		ui_channels[ch_index]:event({ name = "note_off", token = self.token })
		self.token = nil
	end
end

return TestPadView
