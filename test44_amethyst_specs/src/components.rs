use specs::{
    //prelude::*,
    Entity,
    Component, 
    VecStorage,
    NullStorage,
    FlaggedStorage,
    DenseVecStorage
};


#[derive(Debug)]
pub struct PositionComponent {
    pub x: f32,
    pub y: f32,
}

impl PositionComponent{
    pub fn new(x: f32, y: f32) -> Self{
        PositionComponent{
            x,
            y
        }
    }
}

impl Component for PositionComponent {
    type Storage = VecStorage<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Можем описывать компонент с помощью автоматического наследывания
// Указывать тип контейнера компонентов тоже можно
//#[derive(Debug, Component)]
//#[storage(VecStorage)]
#[derive(Debug)]
pub struct VelocityComponent {
    pub x: f32,
    pub y: f32,
}

impl VelocityComponent{
    pub fn new(x: f32, y: f32) -> Self{
        VelocityComponent{
            x,
            y
        }
    }
}

impl Component for VelocityComponent {
    type Storage = VecStorage<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct StoneComponent;

impl Component for StoneComponent {
    // Сам компонент нигде не хранится, а только помечает флагом сущность с компонентом
    type Storage = NullStorage<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TargetComponent {
    pub target: Entity,
    pub offset: [f32; 2],
}

impl Component for TargetComponent {
    // Сам компонент нигде не хранится, а только помечает флагом сущность с компонентом
    type Storage = VecStorage<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DataComponent {
}

impl Component for DataComponent {
    // Специальный тип хранилища для компонента данных, который позволяет подписываться на события
    // изменения данных, обрабатывая только если что-то поменялось
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}