---@class box2 : { pos: vec2, size: vec2 }
---@field __class box2|fun(pos: vec2, size: vec2): box2
---@field overlap fun(self: box2, other: box2): boolean
---@type box2|fun(pos: vec2, size: vec2): box2
box2 = class("box2", {
    ---@param self box2
    ---@param pos vec2
    ---@param size vec2
    __new = function(self, pos, size)
        self.pos = pos
        self.size = size
    end,
    ---@param self box2
    __tostring = function(self)
        return ("[%s, %s]"):format(self.pos, self.size)
    end,
    ---@param self box2
    ---@param other box2
    ---@return boolean
    overlap = function(self, other)
        return
            self.pos.x < other.pos.x + other.size.x and
            self.pos.x + self.size.x > other.pos.x and
            self.pos.y < other.pos.y + other.size.y and
            self.pos.y + self.size.y > other.pos.y
    end,
    ---@param self box2
    ---@param other box2
    ---@return boolean
    __eq = function(self, other)
        if type(other) == "box2" then
            ---@diagnostic disable-next-line: undefined-field
            return self.pos == other.pos and self.size == other.size
        end
        return false
    end
})
