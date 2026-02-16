---@param num integer
---@param size integer
---@return string
function string.intToBits(num, size)
    local s = ""
    while num > 0 do
        local rest = math.fmod(num, 2)
        s = rest .. s
        num = (num - rest) / 2
    end
    return ("0"):rep(size - #s) .. s
end

bit = bit or {}
---@param n integer
---@return integer
function bit.countOne(n)
    local count = 0
    while n > 0 do
        count = count + bit.band(n, 1)
        n = bit.rshift(n, 1)
    end
    return count
end

require "lib.class"
require "lib.vec2"
require "lib.vec3"
require "lib.box2"
require "lib.box3"
