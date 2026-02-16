local SPEED = 5

---@class Player : Entity
---@field qpos vec2
---@type Player|fun(box: box3, qpos: vec2): Player
Player = class("player", {
    ---@param self Player
    ---@param box box3
    ---@param qpos vec2
    __new = function(self, box, qpos)
        Entity.__new(self, box, bit.bor(ENTITY_FLAGS.collidable))
        self.qpos = qpos
    end,
    ---@param self Player
    ---@param state PicoCraftState
    ---@param dt number
    update = function(self, state, dt)
        local move = vec2(
            (love.keyboard.isDown("d") and 1 or 0) - (love.keyboard.isDown("a") and 1 or 0),
            (love.keyboard.isDown("s") and 1 or 0) - (love.keyboard.isDown("w") and 1 or 0)
        ):normal():to3()
        self.vel = move * SPEED
        self:move(state, dt)
    end,
}, Entity)
