use crate::types::{Image, Rect, Vec2i};

use std::rc::Rc;

pub const TILE_SZ: usize = 16;

/// A graphical tile, we'll implement Copy since it's tiny
#[derive(Clone, Copy)]
pub struct Tile {
    pub solid: bool, // ... any extra data like collision flags or other properties
}

/// A set of tiles used in multiple Tilemaps
pub struct Tileset {
    pub tiles: Vec<Tile>,
    image: Rc<Image>,
}

/// Indices into a Tileset
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TileID(usize);

/// Grab a tile with a given ID
impl std::ops::Index<TileID> for Tileset {
    type Output = Tile;
    fn index(&self, id: TileID) -> &Self::Output {
        &self.tiles[id.0]
    }
}
impl Tileset {
    /// Create a new tileset
    pub fn new(tiles: Vec<Tile>, texture: Rc<Image>) -> Self {
        Self {
            tiles,
            image: texture,
        }
    }
    /// Get the frame rect for a tile ID
    fn get_rect(&self, id: TileID) -> Rect {
        let idx = id.0;
        let (w, _h) = self.image.size();
        let tw = w as usize / TILE_SZ;
        let row = idx / tw;
        let col = idx - (row * tw);
        Rect {
            pos: Vec2i {
                x: col as i32 * TILE_SZ as i32,
                y: row as i32 * TILE_SZ as i32,
            },
            sz: Vec2i {
                x: TILE_SZ as i32,
                y: TILE_SZ as i32,
            },
        }
    }
}

/// An actual tilemap
pub struct Tilemap {
    /// Where the tilemap is in space
    pub position: Vec2i,
    /// How big it is
    dims: (usize, usize),
    /// Which tileset is used for this tilemap
    tileset: Rc<Tileset>,
    /// A row-major grid of tile IDs in tileset
    map: Vec<TileID>,
}

impl Tilemap {
    pub fn new(
        position: Vec2i,
        dims: (usize, usize),
        tileset: Rc<Tileset>,
        map: Vec<usize>,
    ) -> Self {
        assert_eq!(dims.0 * dims.1, map.len(), "Tilemap is the wrong size!");
        Self {
            position,
            dims,
            tileset: tileset,
            map: map.into_iter().map(TileID).collect(),
        }
    }

    pub fn tile_id_at(&self, Vec2i { x, y }: Vec2i) -> (Vec2i, TileID) {
        // Translate into map coordinates
        let x = (x - self.position.x) / TILE_SZ as i32;
        let y = (y - self.position.y) / TILE_SZ as i32;
        // return the tile corner and the tile ID
        (
            Vec2i {
                x: x * TILE_SZ as i32 + self.position.x,
                y: y * TILE_SZ as i32 + self.position.y,
            },
            self.map[y as usize * self.dims.0 + x as usize],
        )
    }

    pub fn size(&self) -> (usize, usize) {
        self.dims
    }

    pub fn tile_at(&self, posn: Vec2i) -> (Vec2i, Tile) {
        let (pos, tile_id) = self.tile_id_at(posn);
        (pos, self.tileset[tile_id])
    }

    pub fn draw(&self, screen: &mut Image) {
        for (y, row) in self.map.chunks_exact(self.dims.0).enumerate() {
            // We are in tile coordinates at this point so we'll need to translate back to pixel units and world coordinates to draw.
            let ypx = (y * TILE_SZ) as i32 + self.position.y;
            // Here we can iterate through the column index and tiles in the row in parallel
            for (x, id) in row.iter().enumerate() {
                let xpx = (x * TILE_SZ) as i32 + self.position.x;
                let frame = self.tileset.get_rect(*id);
                screen.bitblt(&self.tileset.image, frame, Vec2i { x: xpx, y: ypx }, false);
            }
        }
    }
}
