rawtype = type
---@param v any
---@return type|string
function type(v)
    if rawtype(v) == "table" then
        local meta = getmetatable(v)
        if rawtype(meta) == "table" then
            if rawtype(meta.__name) == "string" then
                return meta.__name
            end
        else
            return "table"
        end
    end
    return rawtype(v)
end

---@class MetaMethods
---@field __new (fun(self, ...))?
---@field __tostring (fun(self): string)?
---@field __metatable table?
---@field __len (fun(self): number)?
---@field __get (fun(self, k: string|integer): any)?
---@field __set (fun(self, k: string|integer, v))?
---@field __call (fun(self, ...))?
---@field __mode ("v"|"k"|"kv")?
---@field __close (fun(value, err?))?
---@field __gc (fun())?
---@field __pairs (fun(): (fun(value): (fun(state, index: integer): ...), any, integer))?
---@field __ipairs (fun(): (fun(value): (fun(): integer, any), any, integer))?
---@field __add (fun(self, other): any)?
---@field __sub (fun(self, other): any)?
---@field __mul (fun(self, other): any)?
---@field __div (fun(self, other): any)?
---@field __mod (fun(self, other): any)?
---@field __pow (fun(self, other): any)?
---@field __idiv (fun(self, other): any)?
---@field __band (fun(self, other): any)?
---@field __bor (fun(self, other): any)?
---@field __bxor (fun(self, other): any)?
---@field __shl (fun(self, other): any)?
---@field __shr (fun(self, other): any)?
---@field __unm (fun(self): any)?
---@field __bnot (fun(self): any)?
---@field __eq (fun(self, other): boolean)?
---@field __lt (fun(self, other): boolean)?
---@field __le (fun(self, other): boolean)?
---@field __concat (fun(self, other): boolean)?
---@alias ClassMethods MetaMethods|table<string, function>
---@param name string
---@param methods ClassMethods
---@return table
function class(name, methods, ...)
    local derives = { ... }
    local meta = {
        __name = name,
        __index = function(self, k)
            if type(methods.__get) == "function" then
                local value = methods.__get(self, k)
                if value ~= nil then
                    return value
                end
            end
            local value = rawget(self, k)
            if value ~= nil then
                return value
            end
            value = methods[k]
            if value ~= nil then
                return value
            end
            for _, derive in ipairs(derives) do
                value = derive[k]
                if value ~= nil then
                    return value
                end
            end
        end
    }
    for key, value in pairs(methods) do
        if key:sub(1, 2) == "__" then
            meta[key] = value
        end
    end
    return setmetatable(methods, {
        __name = "class<" .. name .. ">",
        __call = function(c, ...)
            local obj = setmetatable({
                __class = c,
                __new = meta.__new,
            }, meta)
            local f = meta.__new
            if type(f) == "function" then
                f(obj, ...)
            end
            return obj
        end,
    })
end
