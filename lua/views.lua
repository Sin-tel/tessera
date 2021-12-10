Views = {
	DefaultView,
	TestView,
}

View = {}

function View:new(name)
	local new = {}
	setmetatable(new,self)
	self.__index = self

	new.super = self
	new.name = name
	return new	
end

function View:draw() end
function View:mousepressed(x, y) end
function View:mouserelease(x, y) end
function View:test() print("super", self.name) end

DefaultView = View:new("default")

function DefaultView:test() 
	print("ah!", self.name) 
	self.super:test()
end