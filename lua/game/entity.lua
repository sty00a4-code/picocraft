ENTITY_FLAGS = {
    collidable = 0b1,
}

---@class Entity : GameObject
---@field __new fun(self: Entity, box: box3, flags: integer)
---@field box box3
---@field vel vec3
---@field flags integer
---@field physics fun(self: Entity, state: State, dt: number)
---@field move fun(self: Entity, state: State, dt: number)
---@field gravity fun(self: Entity, state: State)
---@type Entity|fun(box: box3, flags: integer): Entity
Entity = class("entity", {
    ---@param self Entity
    ---@param box box3
    ---@param flags integer
    __new = function(self, box, flags)
        GameObject.__new(self)
        self.box = box
        self.vel = vec3(0, 0, 0)
        self.flags = flags
    end,
    ---@param self Entity
    ---@param state State
    ---@param dt number
    update = function(self, state, dt)
        self:physics(state, dt)
    end,
    ---@param self Entity
    ---@param state State
    ---@param dt number
    physics = function(self, state, dt)
        self:gravity(state)
        self:move(state, dt)
    end,
    ---@param self Entity
    ---@param state State
    gravity = function(self, state)
        self.vel.z = self.vel.z - GRAVITY
    end,
    ---@param self Entity
    ---@param state State
    ---@param dt number
    move = function(self, state, dt)
        self.box.pos = self.box.pos + self.vel * dt
    end,
    ---@param self Entity
    ---@param state State
    draw = function(self, state)
        love.graphics.setColor(1, 1, 1)
        love.graphics.rectangle("line", self.box.pos.x * TILE_SIZE,
            self.box.pos.y * TILE_SIZE - self.box.pos.z * TILE_SIZE,
            self.box.size.x * TILE_SIZE, self.box.size.y * TILE_SIZE)
    end,
}, GameObject)
