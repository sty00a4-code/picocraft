---@class State : GameObject
---@alias LoadFun fun(self: GameObject, state: State)
---@alias UpdateFun fun(self: GameObject, state: State, dt: number)
---@alias DrawFun fun(self: GameObject, state: State)

---@type fun(state: State)[]
LOAD = {}
---@param f LoadFun
---@param self GameObject
function addLoad(f, self)
    table.insert(LOAD, function(state)
        return f(self, state)
    end)
end

---@type fun(state: State, dt: number)[]
UPDATE = {}
---@param f UpdateFun
---@param self GameObject
function addUpdate(f, self)
    table.insert(UPDATE, function(state, dt)
        return f(self, state, dt)
    end)
end

---@type fun(state: State)[]
DRAW = {}
---@param f DrawFun
---@param self GameObject
function addDraw(f, self)
    table.insert(DRAW, function(state)
        return f(self, state)
    end)
end

---@type LoadFun
local function GOLoad(self, state)
    impl("load")
end
---@type UpdateFun
local function GOUpdate(self, state, dt)
    impl("update")
end
---@type DrawFun
local function GODraw(self, state)
    impl("draw")
end

---@class GameObject
---@field __new fun(self: GameObject)
---@field load LoadFun
---@field update UpdateFun
---@field draw DrawFun
---@type GameObject|fun(): GameObject
GameObject = class("object", {
    __new = function(self)
        if self.load and self.load ~= GOLoad then
            addLoad(self.load, self)
        end
        if self.update and self.update ~= GOUpdate then
            addUpdate(self.update, self)
        end
        if self.draw and self.draw ~= GODraw then
            addDraw(self.draw, self)
        end
    end,
    load = GOLoad,
    update = GOUpdate,
    draw = GODraw,
})
