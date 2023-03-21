#[derive(Clone, PartialEq, Eq)]
pub struct RenderParameters {
    pub wireframe: bool,
    pub colours: bool,
}

pub enum MeshGenerationResult {
    Unbuilt,
    Failure(shared::mesh::CompilationError),
    Successful { triangle_count: usize, volume: f32 },
}

#[derive(Default)]
pub struct OccupiedScreenSpace {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
