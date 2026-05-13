use serde::{Deserialize, Serialize};

/// Módulo de navegación del menú principal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub category: String,
    pub route: String,
    pub parent: Option<String>,
    pub is_favorite: bool,
}

/// Payload para marcar o desmarcar un módulo como favorito.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteToggle {
    pub module_id: String,
    pub favorite: bool,
}
