---@class Camera : GameObject
---@field pos vec3
---@field windowSize vec2
---@field scale number
---@type Camera|fun(pos: vec3, scale: number): Camera
Camera = class("camera", {
    ---@param self Camera
    ---@param pos vec3
    ---@param scale number
    __new = function(self, pos, scale)
        GameObject.__new(self)
        self.pos = pos
        self.windowSize = vec2(0, 0)
        self.scale = scale
    end,
    ---@param self Camera
    ---@param state PicoCraftState
    load = function(self, state)
        local w, h = love.window.getMode()
        self.windowSize = vec2(w, h)
    end,
    ---@param self Camera
    ---@param state PicoCraftState
    ---@param dt number
    update = function(self, state, dt)
        local offset = vec3(self.windowSize.x / TILE_SIZE / 2, self.windowSize.y / TILE_SIZE / 2, 0)
        self.pos = state.player.box.pos - offset
    end,
}, GameObject)
