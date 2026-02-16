---@class TileData
---@field kind "tile"
---@field tile vec2
---@field side vec2

---@class AutoTileData : TileData
---@field kind "autotile"

---@alias AutoTileSide "tl"|"to"|"tr"|"le"|"ri"|"bl"|"bo"|"br"
---@type table<AutoTileSide, integer>
local AUTO_TILING_SIDES = {
    ["tl"] = 0b10000000,
    ["to"] = 0b01000000,
    ["tr"] = 0b00100000,
    ["le"] = 0b00010000,
    ["ri"] = 0b00001000,
    ["bl"] = 0b00000100,
    ["bo"] = 0b00000010,
    ["br"] = 0b00000001,
}
---@type AutoTileSide[]
local AUTO_TILING_ORDER = { "tl", "to", "tr", "le", "ri", "bl", "bo", "br" }
---@param ... AutoTileSide
local function sides2bin(...)
    local bin = 0
    for _, side in ipairs({ ... }) do
        bin = bit.bor(bin, AUTO_TILING_SIDES[side])
    end
    return bin
end
local MASK8 = 0b11111111
---@type vec2[]
local AUTO_TILING_OFFSET = {
    [sides2bin("ri", "bo")] = vec2(0, 0),
    [sides2bin("le", "bo")] = vec2(1, 0),
    [sides2bin("le", "ri")] = vec2(2, 0),
    [sides2bin("le", "ri", "bo")] = vec2(3, 0),
    [sides2bin("le", "ri", "to")] = vec2(4, 0),
    [sides2bin("le")] = vec2(5, 0),
    [sides2bin("to")] = vec2(6, 0),
    [sides2bin("ri", "bo", "br")] = vec2(7, 0),
    [sides2bin("le", "bo", "bl")] = vec2(8, 0),
    [sides2bin("to", "ri")] = vec2(0, 1),
    [sides2bin("to", "le")] = vec2(1, 1),
    [sides2bin("to", "bo")] = vec2(2, 1),
    [sides2bin("to", "bo", "ri")] = vec2(3, 1),
    [sides2bin("to", "bo", "le")] = vec2(4, 1),
    [sides2bin("bo")] = vec2(5, 1),
    [sides2bin("ri")] = vec2(6, 1),
    [sides2bin("to", "tr", "ri")] = vec2(7, 1),
    [sides2bin("to", "tl", "le")] = vec2(8, 1),
    [sides2bin("tl", "to", "tr", "le", "ri", "bl", "bo")] = vec2(0, 2),
    [sides2bin("tl", "to", "tr", "le", "ri", "br", "bo")] = vec2(1, 2),
    [sides2bin("tl", "to", "tr", "le", "ri", "bo")] = vec2(2, 2),
    [sides2bin("to", "le", "ri", "bl", "bo", "br")] = vec2(3, 2),
    [sides2bin("to", "tr", "ri", "bo", "br")] = vec2(4, 2),
    [sides2bin("to", "tl", "le", "bo", "bl")] = vec2(5, 2),
    [MASK8] = vec2(6, 2),
    [sides2bin("to", "le", "ri", "bo")] = vec2(7, 2),
    [sides2bin("tl", "to", "le", "ri", "bl", "bo", "br")] = vec2(0, 3),
    [sides2bin("tr", "to", "le", "ri", "bl", "bo", "br")] = vec2(1, 3),
    [sides2bin("tl", "to", "le", "ri", "bl", "bo")] = vec2(2, 3),
    [sides2bin("tr", "to", "le", "ri", "br", "bo")] = vec2(3, 3),
    [sides2bin("le", "ri", "bl", "bo", "br")] = vec2(4, 3),
    [sides2bin("le", "ri", "tl", "to", "tr")] = vec2(5, 3),
    [0b00000000] = vec2(6, 3),
    [sides2bin("to", "le", "ri", "bo", "br")] = vec2(0, 4),
    [sides2bin("to", "le", "ri", "bo", "bl")] = vec2(1, 4),
    [sides2bin("to", "ri", "bo", "br")] = vec2(2, 4),
    [sides2bin("to", "le", "bo", "bl")] = vec2(3, 4),
    [sides2bin("le", "ri", "bo", "br")] = vec2(4, 4),
    [sides2bin("le", "ri", "bo", "bl")] = vec2(5, 4),
    [sides2bin("tl", "to", "le", "ri", "bo", "br")] = vec2(6, 4),
    [sides2bin("to", "tr", "le", "ri", "bo")] = vec2(0, 5),
    [sides2bin("to", "tl", "le", "ri", "bo")] = vec2(1, 5),
    [sides2bin("to", "tr", "ri", "bo")] = vec2(2, 5),
    [sides2bin("to", "tl", "le", "bo")] = vec2(3, 5),
    [sides2bin("to", "tr", "le", "ri")] = vec2(4, 5),
    [sides2bin("to", "tl", "le", "ri")] = vec2(5, 5),
    [sides2bin("to", "tr", "le", "ri", "bl", "bo")] = vec2(6, 5),
}
-- try to find the closest matching
local function find_closest_mapping(bin)
    local close
    local max = 0
    for id in pairs(AUTO_TILING_OFFSET) do
        if bit.band(bin, id) == id then
            local count = bit.countOne(id)
            if count > max then
                close = id
                max = count
            end
        end
    end
    return close or MASK8
end

local new = {}
for b = 0, MASK8 do
    if not AUTO_TILING_OFFSET[b] then
        local mapped = find_closest_mapping(b)
        new[b] = AUTO_TILING_OFFSET[mapped]
    end
end
for k, v in pairs(new) do
    AUTO_TILING_OFFSET[k] = v
end

---@alias TileDataKind TileData|AutoTileData

---@alias Tile integer
---@alias TileDataList table<Tile, TileDataKind>
---@class TileSet
---@field atlas table
---@field data TileDataList
---@field neighborsToBin fun(self: TileSet, tile: Tile, neighbors: Tile[]): integer
---@field autoTile fun(self: TileSet, tile: Tile, neighbors: Tile[]): vec2 -- neighbors {tl, to, tr, le, ri, bl, bo, br}
---@type TileSet|fun(data: TileDataList, atlas: table): TileSet
TileSet = class("tileset", {
    ---@param self TileSet
    ---@param data TileDataList
    ---@param atlas table
    __new = function(self, data, atlas)
        self.data = data
        self.atlas = atlas
    end,
    ---@param self TileSet
    ---@param tile Tile
    ---@param neighbors Tile[]
    ---@return integer
    neighborsToBin = function(self, tile, neighbors)
        local bin = 0
        for i, ntile in ipairs(neighbors) do
            if ntile == tile then
                local side = AUTO_TILING_ORDER[i]
                bin = bit.bor(bin, AUTO_TILING_SIDES[side])
            end
        end
        return bin
    end,
    ---@param self TileSet
    ---@param tile Tile
    ---@param neighbors Tile[]
    ---@return vec2
    autoTile = function(self, tile, neighbors)
        local bin = self:neighborsToBin(tile, neighbors)
        return self.data[tile].tile + AUTO_TILING_OFFSET[bin]
    end,
})

---@alias Chunk Tile[][][]
---@class TileMap : GameObject
---@field tileset TileSet
---@field chunks Chunk[][]
---@field getChunk fun(self: TileMap, pos: vec2): Chunk?
---@field getTile fun(self: TileMap, pos: vec3): Tile?
---@field setChunk fun(self: TileMap, pos: vec2, chunk: Chunk)
---@field setTile fun(self: TileMap, pos: vec3, tile: Tile)
---@type TileMap|fun(tileset: TileSet): TileMap
TileMap = class("tilemap", {
    ---@param self TileMap
    ---@param tileset TileSet
    __new = function(self, tileset)
        GameObject.__new(self)
        self.tileset = tileset
        self.chunks = {}
    end,
    ---@param self TileMap
    ---@param pos vec2
    ---@return Chunk?
    getChunk = function(self, pos)
        local row = self.chunks[pos.y]
        if not row then
            return
        end
        return row[pos.x]
    end,
    ---@param self TileMap
    ---@param pos vec2
    ---@param chunk Chunk
    setChunk = function(self, pos, chunk)
        local row = self.chunks[pos.y]
        if not row then
            row = {}
            self.chunks[pos.y] = row
        end
        row[pos.x] = chunk
    end,
    ---@param self TileMap
    ---@param pos vec3
    ---@return Tile?
    getTile = function(self, pos)
        local chunkPos = vec2(pos.x / CHUNK_SIZE, pos.y / CHUNK_SIZE):floor()
        local chunk = self:getChunk(chunkPos)
        if not chunk then
            return
        end
        local level = chunk[pos.z]
        if not level then
            return
        end
        local row = level[pos.y]
        if not row then
            return
        end
        return row[pos.x]
    end,
    ---@param self TileMap
    ---@param pos vec3
    ---@param tile Tile
    setTile = function(self, pos, tile)
        local chunkPos = vec2(pos.x / CHUNK_SIZE, pos.y / CHUNK_SIZE):floor()
        local chunk = self:getChunk(chunkPos)
        if not chunk then
            chunk = {}
            self:setChunk(chunkPos, chunk)
        end
        local level = chunk[pos.z]
        if not level then
            level = {}
            chunk[pos.z] = level
        end
        local row = level[pos.y]
        if not row then
            row = {}
            level[pos.y] = row
        end
        row[pos.x] = tile
    end,
    ---@param self TileMap
    ---@param state PicoCraftState
    ---@param dt number
    update = function(self, state, dt)
        local camera = state.camera
        local range = box3(
            camera.pos:floor(),
            (vec3(camera.windowSize.x / TILE_SIZE, camera.windowSize.y / TILE_SIZE, CHUNK_SIZE) / camera.scale):ceil()
        )
        for tz = range.pos.z, range.pos.z + range.size.z do
            for ty = range.pos.y, range.pos.y + range.size.y + range.size.z do
                for tx = range.pos.x, range.pos.x + range.size.x do
                    local pos = vec3(tx, ty, tz)
                    local tile = self:getTile(pos)
                    if tile == nil then
                        local scale = 0.1
                        local height = love.math.noise(tx * scale, ty * scale, tz * scale) * 5 - 1
                        if tz > 0 and tz < height then
                            self:setTile(pos, 1)
                        end
                    end
                end
            end
        end
    end,
    ---@param self TileMap
    ---@param state PicoCraftState
    draw = function(self, state)
        local camera = state.camera
        local range = box3(
            camera.pos:floor(),
            (vec3(camera.windowSize.x / TILE_SIZE, camera.windowSize.y / TILE_SIZE, CHUNK_SIZE) / camera.scale):ceil()
        )
        local tileDatas = self.tileset.data
        local atlas = self.tileset.atlas
        local w, h = atlas:getDimensions()
        for tz = range.pos.z, range.pos.z + range.size.z do
            for ty = range.pos.y, range.pos.y + range.size.y + range.size.z do
                for tx = range.pos.x, range.pos.x + range.size.x do
                    local pos = vec3(tx, ty, tz)
                    local tile = self:getTile(pos)
                    if tile then
                        local data = tileDatas[tile]
                        if data then
                            local quadPos = data.tile * TILE_SIZE
                            if data.kind == "autotile" then
                                local neighbors = {
                                    self:getTile(pos + vec3(-1, -1, 0)) or 0,
                                    self:getTile(pos + vec3(0, -1, 0)) or 0,
                                    self:getTile(pos + vec3(1, -1, 0)) or 0,
                                    self:getTile(pos + vec3(-1, 0, 0)) or 0,
                                    self:getTile(pos + vec3(1, 0, 0)) or 0,
                                    self:getTile(pos + vec3(-1, 1, 0)) or 0,
                                    self:getTile(pos + vec3(0, 1, 0)) or 0,
                                    self:getTile(pos + vec3(1, 1, 0)) or 0,
                                }
                                quadPos = quadPos + self.tileset:autoTile(tile, neighbors) * TILE_SIZE
                            end
                            local quad = love.graphics.newQuad(quadPos.x, quadPos.y, TILE_SIZE,
                                TILE_SIZE, w, h)
                            ---@type vec3
                            local displayPos = pos * TILE_SIZE
                            love.graphics.draw(atlas, quad, displayPos.x, displayPos.y - displayPos.z)
                        end
                    end
                    pos = vec3(tx, ty, tz + 1)
                    tile = self:getTile(pos)
                    if tile and self:getTile(vec3(tx, ty + 1, tz + 1)) == nil then
                        local data = tileDatas[tile]
                        if data then
                            local quadPos = data.tile * TILE_SIZE
                            if data.kind == "autotile" then
                                local neighbors = {
                                    -- top is always present for visuals
                                    tile,
                                    tile,
                                    tile,
                                    self:getTile(pos + vec3(-1, 0, 0)) or 0,
                                    self:getTile(pos + vec3(1, 0, 0)) or 0,
                                    self:getTile(pos + vec3(-1, 0, -1)) or 0,
                                    (self:getTile(pos + vec3(0, 1, -1)) and 0 or self:getTile(pos + vec3(0, 0, -1))) or 0,
                                    self:getTile(pos + vec3(1, 0, -1)) or 0,
                                }
                                local offset = self.tileset:autoTile(tile, neighbors)
                                quadPos = quadPos +
                                    (offset + AUTO_TILE_SIDE_OFFSET) * TILE_SIZE
                            end
                            pos = vec3(tx, ty, tz)
                            local quad = love.graphics.newQuad(quadPos.x, quadPos.y, TILE_SIZE,
                                TILE_SIZE, w, h)
                            ---@type vec3
                            local displayPos = pos * TILE_SIZE
                            love.graphics.draw(atlas, quad, displayPos.x, displayPos.y - displayPos.z)
                        end
                    end
                end
            end
        end
    end
}, GameObject)
