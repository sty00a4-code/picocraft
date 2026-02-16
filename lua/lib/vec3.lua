local sqrt, floor, ceil = math.sqrt, math.floor, math.ceil

---@class vec3 : { x: number, y: number, z: number }
---@field __class vec3|fun(x: number, y: number, z: number): vec3
---@field len fun(self: vec3): number
---@field to2 fun(self: vec3): vec2
---@field normal fun(self: vec3): vec3
---@field floor fun(self: vec3): vec3
---@field ceil fun(self: vec3): vec3
---@type vec3|fun(x: number, y: number, z: number): vec3
vec3 = class("vec3", {
    __new = function(self, x, y, z)
        self.x = x
        self.y = y
        self.z = z
    end,
    __tostring = function(self)
        return ("(%s, %s, %s)"):format(self.x, self.y, self.z)
    end,
    ---@param self vec3
    ---@return number
    len = function(self)
        return sqrt(self.x ^ 2 + self.y ^ 2 + self.z ^ 2)
    end,
    ---@param self vec3
    ---@return vec2
    to2 = function(self)
        return vec2(self.x, self.y)
    end,
    ---@param self vec3
    ---@return vec3
    normal = function(self)
        if self:len() == 0 then
            return vec3(0, 0, 0)
        end
        return self / self:len()
    end,
    ---@param self vec3
    ---@return vec3
    floor = function(self)
        return self.__class(floor(self.x), floor(self.y), floor(self.z))
    end,
    ---@param self vec3
    ---@return vec3
    ceil = function(self)
        return self.__class(ceil(self.x), ceil(self.y), ceil(self.z))
    end,
    ---@param self vec3
    ---@param other vec3
    ---@return vec3
    __add = function(self, other)
        if type(other) ~= "vec3" then
            error("can not perform add on " .. type(self) .. " with " .. type(other), 2)
        end
        ---@diagnostic disable-next-line: undefined-field
        return self.__class(self.x + other.x, self.y + other.y, self.z + other.z)
    end,
    ---@param self vec3
    ---@param other vec3
    ---@return vec3
    __sub = function(self, other)
        if type(other) ~= "vec3" then
            error("can not perform sub on " .. type(self) .. " with " .. type(other), 2)
        end
        ---@diagnostic disable-next-line: undefined-field
        return self.__class(self.x - other.x, self.y - other.y, self.z - other.z)
    end,
    ---@param self vec3
    ---@param other vec3|number
    ---@return vec3
    __mul = function(self, other)
        if type(other) == "vec3" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x * other.x, self.y * other.y, self.z * other.z)
        elseif type(other) == "number" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x * other, self.y * other, self.z * other)
        end
        error("can not perform sub on " .. type(self) .. " with " .. type(other), 2)
    end,
    ---@param self vec3
    ---@param other vec3|number
    ---@return vec3
    __div = function(self, other)
        if type(other) == "vec3" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x / other.x, self.y / other.y, self.z / other.z)
        elseif type(other) == "number" then
            ---@diagnostic disable-next-line: undefined-field
            return self.__class(self.x / other, self.y / other, self.z / other)
        end
        error("can not perform sub on " .. type(self) .. " with " .. type(other), 2)
    end,
    ---@param self vec3
    ---@param other vec3|number
    ---@return boolean
    __eq = function(self, other)
        if type(other) == "vec3" then
            ---@diagnostic disable-next-line: undefined-field
            return self.x == other.x and self.y == other.y and self.z == other.z
        end
        return false
    end,
})
