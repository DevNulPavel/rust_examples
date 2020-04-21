use specs::{
    prelude::*,
    Join,
    Read,
    ReadStorage,
    System, 
    WriteStorage,
    Entities,
    LazyUpdate
};
use crate::{
    components::*,
    resources::*
};

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct HelloWorldSystem;

impl<'a> System<'a> for HelloWorldSystem {
    type SystemData = ReadStorage<'a, PositionComponent>;
    
    fn run(&mut self, position: Self::SystemData) {        
        for position in position.join() {
            println!("Hello, {:?}", &position);
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UpdatePosSystem;

impl<'a> System<'a> for UpdatePosSystem {
    type SystemData = (Read<'a, DeltaTime>,
                       ReadStorage<'a, VelocityComponent>,
                       WriteStorage<'a, PositionComponent>);
    
    fn run(&mut self, (time, vel, mut pos): Self::SystemData) {
        // Находим все сущности, которые содержат и ускорение, и позицию
        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x * time.time;
            pos.y += vel.y * time.time;
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Можно определить собственный класс системных данных
#[derive(SystemData)]
pub struct StoneCreatorSystemData<'a> {
    entities: Entities<'a>,
    stones: WriteStorage<'a, StoneComponent>,
    updater: Read<'a, LazyUpdate>

    // positions: ReadStorage<'a, Position>,
    // velocities: ReadStorage<'a, Velocity>,
    // forces: ReadStorage<'a, Force>,
    // delta: Read<'a, DeltaTime>,
    // game_state: Write<'a, GameState>,
}

#[derive(Default)]
pub struct StoneCreatorSystem{
    inserted: bool
}

impl<'a> System<'a> for StoneCreatorSystem {
    // Можно определить собственный класс системных данных
    type SystemData = StoneCreatorSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if self.inserted {
            return;
        }

        self.inserted = true;

        // Создаем сущность камная
        let stone = data.entities.create();

        // Либо мы добавляем компонент на камень через хранилище компонентов камней
        // Но это вызывает блокировку и синхронизацию, лучше делать это отложенно
        data.stones.insert(stone, StoneComponent).unwrap();

        // Либо мы можем отложенно добавить компонент с помощью LazyUpdate, к
        // омпонент добавится в world.maintain() после кадра
        data.updater.insert(stone, StoneComponent);
        data.updater.insert(stone, VelocityComponent::new(0.1, 0.1));
        data.updater.insert(stone, PositionComponent::new(0.0, 0.0));
    }
}