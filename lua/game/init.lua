require "game.tilemap"
require "game.entity"
require "game.camera"
require "game.player"
GRAVITY = 10
TILE_SIZE = 32
CHUNK_SIZE = 16
AUTO_TILE_OFFSET = vec2(0, 12)
AUTO_TILE_SIDE_OFFSET = vec2(0, 6)

---@type integer[]
local LAST_DELETED = {}
---@class PicoCraftState : State
---@field atlas table
---@field camera Camera
---@field map TileMap
---@field player Player
---@field entities Entity[]
---@field spawn fun(self: PicoCraftState, entity: Entity): integer
---@type PicoCraftState|fun(): PicoCraftState
PicoCraftState = class("picocraft-state", {
    ---@param self PicoCraftState
    __new = function(self)
        GameObject.__new(self)
        self.camera = Camera(vec3(0, 0, 0), 1)
        self.map = TileMap(TileSet({
            [1] = { -- grass
                kind = "autotile",
                tile = vec2(0, 12 * 0),
                side = vec2(0, 12 * 0) + AUTO_TILE_SIDE_OFFSET,
            },
            [2] = {
                kind = "autotile",
                tile = vec2(0, 12 * 0) + AUTO_TILE_SIDE_OFFSET,
                side = vec2(0, 12 * 0) + AUTO_TILE_SIDE_OFFSET,
            },
            [3] = {
                kind = "autotile",
                tile = vec2(0, 12 * 1),
                side = vec2(0, 12 * 1) + AUTO_TILE_SIDE_OFFSET,
            },
        }, {}))
        self.player = Player(box3(vec3(0, 0, 0), vec3(1, 1, 1)), vec2(9, 0))
        self.entities = {
            self.player
        }
    end,
    ---@param self PicoCraftState
    load = function(self)
        self.atlas = love.graphics.newImage("assets/tileset.png")
        self.map.tileset.atlas = self.atlas
    end,
    ---@param self PicoCraftState
    ---@param entity Entity
    spawn = function(self, entity)
        local id = table.remove(LAST_DELETED)
        if id then
            self.entities[id] = entity
        else
            table.insert(self.entities, entity)
        end
    end,
    ---@param self PicoCraftState
    ---@param id integer
    despawn = function(self, id)
        self.entities[id] = nil
        table.insert(LAST_DELETED, id)
    end,
}, GameObject)

local state = PicoCraftState()

function love.load()
    love.graphics.setDefaultFilter("nearest", "nearest")
    love.window.setTitle("PicoCraft")
    for _, load in ipairs(LOAD) do
        load(state)
    end
end

function love.update(dt)
    for _, update in ipairs(UPDATE) do
        update(state, dt)
    end
end

function love.draw()
    love.graphics.clear(0, 0.5, 1)
    love.graphics.push()
    love.graphics.scale(state.camera.scale)
    love.graphics.translate(-state.camera.pos.x * TILE_SIZE, -(state.camera.pos.y - state.camera.pos.z) * TILE_SIZE)
    for _, draw in ipairs(DRAW) do
        draw(state)
    end
    for _, entity in ipairs(state.entities) do
        entity:draw(state)
    end
    love.graphics.pop()
end
