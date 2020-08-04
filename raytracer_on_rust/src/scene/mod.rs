mod intersection;
mod intersection_full;
mod scene;

// Экспортировать можно с помощью self из текущего модуля
pub(crate) use self::{
    scene::{
        Scene,
        build_test_scene
    },
    intersection::{
        Intersection
    },
    // intersection_full::{
    //     IntersectionFull
    // }
};