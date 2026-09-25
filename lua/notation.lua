local log = require("log")

-- Only handles coordinates -> string representation for note font, doesn't know anything about tuning

local Notation = {}

-- circle of fifths
local NOMINALS = { "F", "C", "G", "D", "A", "E", "B" }
local N_NOMINALS = #NOMINALS

-- available accidentals in the font
Notation.ACC_UP = { "w", "v" }
Notation.ACC_SEPTIMAL = { "o", "n" }
Notation.ACC_UNDECIMAL = { "h", "e" }
Notation.ACC_ARROWS = { "r", "s" }

local ACC_FLAT = "a"
-- local ACC_NATURAL = "b" -- unused
local ACC_SHARP = "c"
local ACC_DOUBLE_SHARP = "d"
local ACC_HALF_FLAT = "e"
local ACC_HALF_SHARP = "f"
local ACC_SESQUI_FLAT = "g" -- one-and-a-half flat

local ACC_JOHNSTON_PLUS = { "l", "m" }
local ACC_JOHNSTON_SEPTIMAL = { "p", "q" }
local ACC_JOHNSTON_UNDECIMAL = Notation.ACC_ARROWS

local function accidental(n, acc)
	if n > 0 then
		return string.rep(acc[1], n)
	elseif n < 0 then
		return string.rep(acc[2], -n)
	else
		return ""
	end
end

-- sharps/flats glyphs and double sharp ligature
local function sharps_str(sharps)
	local acc = ""
	if sharps > 0 then
		if sharps % 2 == 1 then
			acc = acc .. ACC_SHARP
		end
		acc = acc .. string.rep(ACC_DOUBLE_SHARP, math.floor(sharps / 2))
	elseif sharps < 0 then
		acc = acc .. string.rep(ACC_FLAT, -sharps)
	end
	return acc
end

Notation.__index = Notation
-- style is a NotationStyle, see tuning.rs
function Notation.new(style)
	local self = setmetatable({}, Notation)

	self.accidentals = {}
	-- coordinate of the half sharp
	self.half_sharp = nil

	-- accidentals start after the octave and fifth
	local offset = 2

	for i, acc in ipairs(style.accidentals) do
		if acc.half_sharp then
			self.half_sharp = i + offset
		end
	end
	self.johnston = style.johnston

	if style.johnston then
		for i, acc in ipairs(style.accidentals) do
			if not acc.half_sharp then
				if acc.ratio == "81/80" then
					table.insert(self.accidentals, { i + offset, 5 })
				elseif acc.ratio == "64/63" then
					table.insert(self.accidentals, { i + offset, 7 })
				elseif acc.ratio == "33/32" then
					table.insert(self.accidentals, { i + offset, 11 })
				else
					log.error("Unknown accidental: " .. acc.ratio)
				end
			end
		end
	else
		if #style.accidentals == 1 and not self.half_sharp then
			-- single is up/down
			table.insert(self.accidentals, { 1 + offset, Notation.ACC_UP })
		else
			for i, acc in ipairs(style.accidentals) do
				if not acc.half_sharp then
					if acc.ratio == "81/80" then
						table.insert(self.accidentals, { i + offset, Notation.ACC_UP })
					elseif acc.ratio == "64/63" then
						table.insert(self.accidentals, { i + offset, Notation.ACC_SEPTIMAL })
					elseif acc.ratio == "33/32" then
						table.insert(self.accidentals, { i + offset, Notation.ACC_UNDECIMAL })
					else
						log.error("Unknown accidental: " .. acc.ratio)
					end
				end
			end
		end
	end

	return self
end

-- get nominals and sharps from basic chain of fifths
function Notation:get_nominal(p)
	local fifths = (p[2] or 0) + 1

	local nominal_offset = fifths % N_NOMINALS

	-- offset 1 to get relative to C
	local nominal = NOMINALS[nominal_offset + 1]
	local sharps = math.floor(fifths / N_NOMINALS)

	return nominal, sharps, nominal_offset
end

function Notation:get_name_johnston(p)
	local nominal, sharps, nominal_offset = self:get_nominal(p)

	local acc = sharps_str(sharps)

	-- 25/24 = vv#
	local plus = sharps * 2

	-- A, E, B need extra +
	if nominal_offset > 3 then
		plus = plus + 1
	end

	for _, v in ipairs(self.accidentals) do
		local index, prime = v[1], v[2]
		if prime == 7 then
			-- 36/35 = ^7
			acc = acc .. accidental(p[index], ACC_JOHNSTON_SEPTIMAL)
			plus = plus - p[index]
		elseif prime == 11 then
			-- 33/32 works the same
			acc = acc .. accidental(p[index], ACC_JOHNSTON_UNDECIMAL)
		end
	end

	-- need to handle +/- last since they get modified by ones above
	for _, v in ipairs(self.accidentals) do
		local index, prime = v[1], v[2]
		if prime == 5 then
			-- 81/80 = +
			plus = plus + p[index]
			acc = acc .. accidental(plus, ACC_JOHNSTON_PLUS)
		end
	end

	return nominal .. acc
end

function Notation:name(p)
	if self.johnston then
		return self:get_name_johnston(p)
	end

	local nominal, sharps, _ = self:get_nominal(p)

	local acc = ""
	local acc_pre = ""

	if self.half_sharp then
		-- half sharp exception:
		-- ligature with the sharps because two of them are a sharp
		local hsharp = sharps * 2 + p[self.half_sharp]
		if hsharp >= 0 then
			if hsharp % 4 == 3 then
				acc = ACC_SESQUI_FLAT
				sharps = math.floor(hsharp / 2) - 1
			else
				if hsharp % 2 == 1 then
					acc = ACC_HALF_SHARP
				end
				sharps = math.floor(hsharp / 2)
			end
		else
			local hflat = -hsharp
			if hflat % 2 == 1 then
				acc = ACC_HALF_FLAT
			end
			sharps = -math.floor(hflat / 2)
		end
	end

	acc = acc .. sharps_str(sharps)

	for _, v in ipairs(self.accidentals) do
		local index, accidental_pair = v[1], v[2]
		-- put ups/downs in front
		if accidental_pair[1] == "w" then
			acc_pre = acc_pre .. accidental(p[index], accidental_pair)
		else
			acc = acc .. accidental(p[index], accidental_pair)
		end
	end

	return acc_pre .. nominal .. acc
end

return Notation
