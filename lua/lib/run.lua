-- require("main")

function love.run()

	if love.load then love.load(love.arg.parseGameArguments(arg), arg) end
 
	-- We don't want the first frame's dt to include time taken by love.load.
	if love.timer then love.timer.step() end
 
	local dt = 0

	local accum = 0

	local time = love.timer.getTime( )

 
	-- Main loop time.
	return function()
		-- Process events.
		if love.event then
			love.event.pump()
			for name, a,b,c,d,e,f in love.event.poll() do
				if name == "quit" then
					if not love.quit or not love.quit() then
						return a or 0
					end
				end
				-- print(name)
				love.handlers[name](a,b,c,d,e,f)
			end
		end

		-- do one step to ignore the time spent drawing previous frame
		if love.timer then love.timer.step() end

		accum = 0
 		
 		while true do
			if love.timer then dt = love.timer.step() end

			accum = accum + dt
	 		
	 		local time2 = love.timer.getTime( )
	 		-- print(1/(time2 - time))
	 		

	 		if love.update then love.update(time2 - time) end 

	 		time = time2

	 		if accum >= 1/60 then
	 			break
	 		end

	 		if love.timer then love.timer.sleep(0.002) end
	 	end

		-- Call draw
		if love.graphics and love.graphics.isActive() then
			love.graphics.origin()
			love.graphics.clear(love.graphics.getBackgroundColor())
 
			if love.draw then love.draw() end

			-- print("present")
			love.graphics.present()
		end
 
		
	end
end