---@class box3 : { pos: vec3, size: vec3 }
---@field __class box3|fun(pos: vec3, size: vec3): box3
---@type box3|fun(pos: vec3, size: vec3): box3
box3 = class("box3", {
    ---@param self box3
    ---@param pos vec3
    ---@param size vec3
    __new = function(self, pos, size)
        self.pos = pos
        self.size = size
    end,
    ---@param self box3
    __tostring = function(self)
        return ("[%s, %s]"):format(self.pos, self.size)
    end,
    ---@param self box3
    ---@param other box3
    ---@return boolean
    overlap = function(self, other)
        return
            self.pos.x < other.pos.x + other.size.x and
            self.pos.x + self.size.x > other.pos.x and
            self.pos.y < other.pos.y + other.size.y and
            self.pos.y + self.size.y > other.pos.y and
            self.pos.z < other.pos.z + other.size.z and
            self.pos.z + self.size.z > other.pos.z
    end,
    ---@param self box3
    ---@param other box3
    ---@return boolean
    __eq = function(self, other)
        if type(other) == "box3" then
            ---@diagnostic disable-next-line: undefined-field
            return self.pos == other.pos and self.size == other.size
        end
        return false
    end
})
