local build = require("build")
local load_default_project = require("default.project")
local tuning = require("tuning")

local command = {}

command.max_size = 50
command.stack = {}
command.index = 0

-- Edits in progress, that register their command only when they finish: tool drags, recording.
-- Until then the project doesn't match the top of the stack, so anything that undoes, redoes or
-- replaces the project has to finish them first (see finish_edits).
-- owner -> function that finishes the edit
local pending = {}

-- `finish` should register the command and end the edit.
function command.begin_edit(owner, finish)
	pending[owner] = finish
end

function command.end_edit(owner)
	pending[owner] = nil
end

function command.finish_edits()
	for owner, finish in pairs(pending) do
		pending[owner] = nil
		finish()
	end
	assert(next(pending) == nil, "edit started while finishing edits")
end

-- Debug check that the project is consistent with the tuning, after every change to the stack.
local function validate()
	if release then
		return
	end
	tuning.validate()
end

function command.run_and_register(c)
	c:run()
	command.register(c)
end

-- Commands can have an `is_empty` method: when it's true the command changed nothing, so it's dropped
-- instead of adding a step that does nothing to the undo stack.
function command.register(c)
	if c.is_empty and c:is_empty() then
		return
	end

	command.index = command.index + 1

	-- remove irrelevant future commands
	for i = #command.stack, command.index, -1 do
		command.stack[i] = nil
	end

	command.stack[command.index] = c

	-- remove events exceeding max size
	if #command.stack > command.max_size then
		table.remove(command.stack, 1)
		command.index = command.index - 1
	end
	validate()
end

function command.undo()
	command.finish_edits()
	if command.index >= 1 then
		command.stack[command.index]:reverse()
		command.index = command.index - 1
	else
		command.index = 0
		print("nothing to undo!")
	end
	validate()
end

function command.redo()
	command.finish_edits()
	command.index = command.index + 1
	if command.stack[command.index] then
		command.stack[command.index]:run()
	else
		command.index = #command.stack
		print("nothing to redo!")
	end
	validate()
end

-- a change to some variable
local Change = {}
Change.__index = Change

function Change.new(target, key, value)
	local self = setmetatable({}, Change)

	assert(type(target) == "table")
	assert(value ~= nil)
	self.target = target
	self.key = key
	self.value = value
	self.prev_value = target[key]

	return self
end

function Change:run()
	self.target[self.key] = self.value
end

function Change:reverse()
	self.target[self.key] = self.prev_value
end

function Change:is_empty()
	return self.value == self.prev_value
end

command.Change = Change

--
local NewProject = {}
NewProject.__index = NewProject

function NewProject.new()
	local self = setmetatable({}, NewProject)

	command.finish_edits()
	self.prev = util.clone(project)
	return self
end

function NewProject:run()
	-- cleanup current project
	build.new_project()
	load_default_project()
end

function NewProject:reverse()
	build.load_project(util.clone(self.prev))
end

command.NewProject = NewProject

--
local NoteUpdate = {}
NoteUpdate.__index = NoteUpdate

function NoteUpdate.new(prev_state, new_state)
	local self = setmetatable({}, NoteUpdate)

	assert(prev_state)
	assert(new_state)

	-- keep ref to actual notes, state is just copies
	self.notes = {}
	for _, v in ipairs(new_state) do
		table.insert(self.notes, v)
	end

	self.prev_state = prev_state
	self.new_state = util.clone(new_state)

	return self
end

function NoteUpdate:run()
	for i, v in ipairs(self.notes) do
		for key in pairs(v) do
			self.notes[i][key] = self.new_state[i][key]
		end
	end
end

function NoteUpdate:reverse()
	for i, v in ipairs(self.notes) do
		for key in pairs(v) do
			self.notes[i][key] = self.prev_state[i][key]
		end
	end
end

function NoteUpdate:is_empty()
	return util.deep_eq(self.prev_state, self.new_state)
end

command.NoteUpdate = NoteUpdate

--
local NoteDelete = {}
NoteDelete.__index = NoteDelete

function NoteDelete.new(notes)
	local self = setmetatable({}, NoteDelete)

	assert(notes)
	self.notes = notes
	self.mask = {}

	for ch_index in pairs(self.notes) do
		for _, note in ipairs(self.notes[ch_index]) do
			self.mask[note] = true
		end
	end
	return self
end

function NoteDelete:run()
	-- remove selected notes
	for _, channel in ipairs(project.channels) do
		if channel.notes then
			for i = #channel.notes, 1, -1 do
				if self.mask[channel.notes[i]] then
					table.remove(channel.notes, i)
				end
			end
		end
	end
end

function NoteDelete:reverse()
	for ch_index in pairs(self.notes) do
		for _, note in ipairs(self.notes[ch_index]) do
			table.insert(project.channels[ch_index].notes, note)
		end
	end
end

function NoteDelete:is_empty()
	return next(self.mask) == nil
end

command.NoteDelete = NoteDelete

--
local NoteAdd = {}
NoteAdd.__index = NoteAdd

function NoteAdd.new(notes)
	local self = setmetatable({}, NoteAdd)

	self.notes = notes
	self.mask = {}

	-- use pairs because notes may be sparse
	for ch_index in pairs(self.notes) do
		for _, note in ipairs(self.notes[ch_index]) do
			self.mask[note] = true
		end
	end
	return self
end

function NoteAdd:run()
	for ch_index in pairs(self.notes) do
		for _, note in ipairs(self.notes[ch_index]) do
			table.insert(project.channels[ch_index].notes, note)
		end
	end
end

function NoteAdd:reverse()
	for _, channel in ipairs(project.channels) do
		if channel.notes then
			for i = #channel.notes, 1, -1 do
				if self.mask[channel.notes[i]] then
					table.remove(channel.notes, i)
				end
			end
		end
	end
end

function NoteAdd:is_empty()
	return next(self.mask) == nil
end

command.NoteAdd = NoteAdd

--
local TimeUpdate = {}
TimeUpdate.__index = TimeUpdate

function TimeUpdate.new(prev_state, new_state)
	local self = setmetatable({}, TimeUpdate)

	assert(prev_state)
	assert(new_state)

	self.prev_state = util.clone(prev_state)
	self.new_state = util.clone(new_state)

	return self
end

function TimeUpdate:run()
	project.time = self.new_state
end

function TimeUpdate:reverse()
	project.time = self.prev_state
end

function TimeUpdate:is_empty()
	return util.deep_eq(self.prev_state, self.new_state)
end

command.TimeUpdate = TimeUpdate

-- Change the project's tuning, which converts every note.
local SetTuning = {}
SetTuning.__index = SetTuning

function SetTuning.new(def)
	local self = setmetatable({}, SetTuning)

	-- edits in progress can hold notes that aren't converted
	command.finish_edits()
	self.def = util.clone(def)
	self.prev_def = util.clone(project.settings.tuning)
	self.prev_center = util.clone(project.center)

	-- keep refs to the notes, and copies of their intervals
	self.notes = tuning.all_notes()
	self.prev_intervals = {}
	for i, note in ipairs(self.notes) do
		self.prev_intervals[i] = util.clone(note.interval)
	end

	return self
end

function SetTuning:run()
	return tuning.set(util.clone(self.def))
end

function SetTuning:reverse()
	assert(tuning.load(util.clone(self.prev_def)))
	project.settings.tuning = util.clone(self.prev_def)
	for i, note in ipairs(self.notes) do
		note.interval = util.clone(self.prev_intervals[i])
	end
	project.center = util.clone(self.prev_center)
	clipboard.clear()
end

command.SetTuning = SetTuning

-- Move the tuning center ('c' in the canvas).
-- Notes are untouched, only how they're named and the grid move.
local SetCenter = {}
SetCenter.__index = SetCenter

function SetCenter.new(center)
	local self = setmetatable({}, SetCenter)

	self.center = util.clone(center)
	self.prev_center = util.clone(tuning.get_center())

	return self
end

function SetCenter:run()
	tuning.set_center(self.center)
end

function SetCenter:reverse()
	tuning.set_center(self.prev_center)
end

function SetCenter:is_empty()
	return tuning.eq(self.center, self.prev_center)
end

command.SetCenter = SetCenter

command.NewChannel, command.RemoveChannel, command.NewEffect, command.RemoveEffect, command.ReorderEffect =
	unpack(require("command_channel"))

return command
