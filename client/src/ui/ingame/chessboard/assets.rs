use bevy::asset::io::embedded::GetAssetServer;
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource to hold all chess piece textures.
#[derive(Resource)]
pub struct ChessAssets {
    piece_textures: HashMap<char, Handle<Image>>,
}

impl ChessAssets {
    pub fn get(&self, id: char) -> Option<&Handle<Image>> {
        self.piece_textures.get(&id)
    }
}

/// FromWorld is a mighty trait from bevy that essentially allows us to initialize a resource with
/// direct access to the whole bevy system. The bevy cookbook mentions that it shouldn't be overused
/// as it could easily lead to lots of spaghetti. We use it here, because ChessAssets needs
/// access to the asset_server during its default initialization when we do `.init_resource()`
/// in the plugin app builder.
impl FromWorld for ChessAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_asset_server();
        let mut piece_textures = HashMap::new();
        let pieces = [
            ('r', "r_b.png"),
            ('n', "n_b.png"),
            ('b', "b_b.png"),
            ('q', "q_b.png"),
            ('k', "k_b.png"),
            ('p', "p_b.png"),
            ('R', "r_w.png"),
            ('N', "n_w.png"),
            ('B', "b_w.png"),
            ('Q', "q_w.png"),
            ('K', "k_w.png"),
            ('P', "p_w.png"),
        ];
        for (key, path) in pieces {
            piece_textures.insert(key, asset_server.load(path.to_string()));
        }

        ChessAssets { piece_textures }
    }
}
