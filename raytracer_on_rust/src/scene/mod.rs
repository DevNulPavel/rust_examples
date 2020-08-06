mod intersection;
mod intersection_full;
mod scene;
mod test_scene;

// Экспортировать можно с помощью self из текущего модуля
pub(crate) use self::{
    scene::{
        Scene
    },
    test_scene::{
        build_test_scene
    },
    intersection::{
        Intersection
    }
};