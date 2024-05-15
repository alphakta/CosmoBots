struct World {
    // Vous pouvez ajouter ici des propriétés pour gérer l'état de votre monde si nécessaire
}

impl World {
    fn new() -> Self {
        Self {
            // Initialisation
        }
    }

    fn spawn_elements(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) {
        // Utilisez asset_server pour charger des textures et commands pour créer des entités avec ces textures
        // Exemple : Création d'un rocher
        let rock_texture_handle = asset_server.load("chemin/vers/texture/rocher.png");
        commands.spawn_bundle(SpriteBundle {
            texture: rock_texture_handle,
            ..default()
        }).insert(Rock);

        // Répétez pour Water et Grass...
    }
}