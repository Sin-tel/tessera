ChannelView =  View:derive("Channels")

function ChannelView:new()
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.select = nil

	new.scroll = 0
	new.scroll_ = 0

	return new
end

function ChannelView:update()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	self.scroll = self.scroll + (self.scroll_ - self.scroll)*0.5
	self.scroll = clamp(self.scroll, 0, self:getMaxScroll())
	self.scroll_ = clamp(self.scroll_, 0, self:getMaxScroll())


	if math.abs(self.scroll - self.scroll_) < 2 then
		self.scroll = self.scroll_
	end
	

	if self.box.focus then
		self.index = math.floor((my + self.scroll)/UI_GRID) + 1
	end
end

function ChannelView:mousepressed()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	if Mouse.button == 1 then
		for i,v in ipairs(channels.list) do
			if i == self.index then
				if mx < w - BUTTON_SMALL*5 then
					selection.channel = v
				else
					local ind = math.floor(((mx - w)/BUTTON_SMALL) + 6)
					if ind == 1 then
						channels.mute(v, not v.mute)
						for _,ch in ipairs(channels.list) do
							ch.solo = false
						end
					elseif ind == 2 then
						if v.solo then
							for _,ch in ipairs(channels.list) do
								ch.solo = false
								channels.mute(ch, false)
							end
						else
							for _,ch in ipairs(channels.list) do
								ch.solo = false
								channels.mute(ch, true)
							end
							v.solo = true
							channels.mute(v, false)
						end
					elseif ind == 3 then
						if v.armed then
							v.armed = false
						else
							for _,ch in ipairs(channels.list) do
								ch.armed = false
							end
							v.armed = true
						end
					elseif ind == 4 then
						v.visible = not v.visible
					elseif ind == 5 then
						v.lock = not v.lock
					end
				end


			end
		end
	end
end


function ChannelView:draw()
	local w, h = self:getDimensions()
	local mx, my = self:getMouse()

	for i, v in ipairs(channels.list) do
		local y = UI_GRID*(i-1) - self.scroll

		if self.index == i then
			love.graphics.setColor(Theme.bg_highlight)
			love.graphics.rectangle("fill", 0, y, w, UI_GRID)
		end

		love.graphics.setColor(Theme.ui_text)
		if selection.channel == v then
			love.graphics.setColor(Theme.highlight)
		end
		drawText(v.name, 0, y, w - BUTTON_SMALL*5 , UI_GRID, "left")

		if v.mute then
			love.graphics.setColor(Theme.mute)
		else
			love.graphics.setColor(Theme.text_dim)
		end
		love.graphics.draw(icons.mute   , w - BUTTON_SMALL*5, y+4)

		if v.solo then
			love.graphics.setColor(Theme.solo)
		else
			love.graphics.setColor(Theme.text_dim)
		end
		love.graphics.draw(icons.solo   , w - BUTTON_SMALL*4, y+4)

		if v.armed then
			love.graphics.setColor(Theme.recording)
		else
			love.graphics.setColor(Theme.text_dim)
		end
		love.graphics.draw(icons.armed  , w - BUTTON_SMALL*3, y+4)

		if v.visible then
			love.graphics.setColor(Theme.ui_text)
			love.graphics.draw(icons.visible, w - BUTTON_SMALL*2, y+4)
		else
			love.graphics.setColor(Theme.text_dim)
			love.graphics.draw(icons.invisible, w - BUTTON_SMALL*2, y+4)
		end
		
		if v.lock then
			love.graphics.setColor(Theme.ui_text)
			love.graphics.draw(icons.lock, w - BUTTON_SMALL*1, y+4)
		else
			love.graphics.setColor(Theme.text_dim)
			love.graphics.draw(icons.unlock, w - BUTTON_SMALL*1, y+4)
		end

	end
end

function ChannelView:wheelmoved(y)
	self.scroll_ = math.floor(self.scroll - y*1.5*UI_GRID)
end

function ChannelView:getMaxScroll()
	local w, h = self:getDimensions()

	local l = #channels.list
	return math.max(0, (l*UI_GRID) - h)
end