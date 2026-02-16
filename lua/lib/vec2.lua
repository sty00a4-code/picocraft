local sqrt, floor, ceil = math.sqrt, math.floor, math.ceil

---@class vec2 : { x: number, y: number }
---@field __class vec2|fun(x: number, y: number): vec2
---@field len fun(self: vec2): number
---@field to3 fun(self: vec2): vec3
---@field normal fun(self: vec2): vec2
---@field floor fun(self: vec2): vec2
---@field ceil fun(self: vec2): vec2
---@type vec2|fun(x: number, y: number): vec2
vec2 = class("vec2", {
    __new = function(self, x, y)
        self.x = x
        self.y = y
    end,
    __tostring = function(self)
        return ("(%s, %s)"):format(self.x, self.y)
    end,
    ---@param self vec2
    ---@return number
    len = function(self)
        return sqrt(self.x ^ 2 + self.y ^ 2)
    end,
    ---@param self vec2
    ---@return vec3
    to3 = function(self)
        return vec3(self.x, self.y, 0)
    end,
    ---@param self vec2
    ---@return vec2
    normal = function(self)
        if self:len() == 0 then
            return vec2(0, 0)
        end
        return self / self:len()
    end,
    ---@param self vec2
    ---@return vec2
    floor = function(self)
        return self.__class(floor(self.x), floor(self.y))
    end,
    ---@param self vec2
    ---@return vec2
    ceil = function(self)
        return self.__class(ceil(self.x), ceil(self.y))
    end,
    ---@param self vec2
    ---@param other vec2
    ---@return vec2
    __add = function(self, other)
        if type(other) ~= "vec2" then
            error("can not perform add on " .. type(self) .. " with " .. type(other), 2)
        end
        ---@diagnostic disable-next-line: undefined-field
        return self.__class(self.x + other.x, self.y + other.y)
    end,
    ---@param self vec2
    ---@param other vec2
    ---@return vec2
    __sub = function(self, other)
        if type(other) ~= "vec2" then
            error("can not perform sub on " .. type(self) .. " with " .. type(other), 2)
        end
        ---@diagnostic disable-next-line: undefined-field
        return self.__class(self.x - other.x, self.y - other.y)
    end,
    ---@param self vec2
    ---@param other vec2|number
    ---@return vec2
    __mul = function(self, other)
        if type(other) == "vec2" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x * other.x, self.y * other.y)
        elseif type(other) == "number" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x * other, self.y * other)
        end
        error("can not perform sub on " .. type(self) .. " with " .. type(other), 2)
    end,
    ---@param self vec2
    ---@param other vec2|number
    ---@return vec2
    __div = function(self, other)
        if type(other) == "vec2" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x / other.x, self.y / other.y)
        elseif type(other) == "number" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x / other, self.y / other)
        end
        error("can not perform sub on " .. type(self) .. " with " .. type(other), 2)
    end,
    ---@param self vec2
    ---@param other vec2|number
    ---@return boolean
    __eq = function(self, other)
        if type(other) == "vec2" then
            ---@diagnostic disable-next-line: undefined-field
            return self.x == other.x and self.y == other.y
        end
        return false
    end,
})
