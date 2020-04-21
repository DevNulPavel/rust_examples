use specs::{
    prelude::*,
    Join,
    Read,
    ReadStorage,
    System, 
    WriteStorage,
    Entities,
    LazyUpdate,
    shrev::{
        ReaderId,
        EventChannel,
    }
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
            println!("Hello-> {:?}", &position);
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UpdatePosSystem;

impl<'a> System<'a> for UpdatePosSystem {
    type SystemData = (Read<'a, DeltaTime>,
                       ReadStorage<'a, VelocityComponent>,
                       WriteStorage<'a, PositionComponent>,
                       Write<'a, EventChannel<AppEvent>>);
    
    fn run(&mut self, (time, vel, mut pos, mut events): Self::SystemData) {
        // Находим все сущности, которые содержат и ускорение, и позицию
        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x * time.time;
            pos.y += vel.y * time.time;
        }

        // Отправка сообщения в канал
        events.single_write(AppEvent::Moved);
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Можно определить собственный класс системных данных
#[derive(SystemData)]
pub struct StoneCreatorSystemData<'a> {
    entities: Entities<'a>,
    stones: WriteStorage<'a, StoneComponent>,
    updater: Read<'a, LazyUpdate>,
    events_channel: Write<'a, EventChannel<AppEvent>>

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

        // Есть ресурс канала, мы можем по каналу отправлять сообщения другим системам
        data.events_channel.single_write(AppEvent::Created);
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AppEvent{
    Created,
    Moved,
}

#[derive(Default)]
pub struct EventProcessSystem {
    reader: Option<ReaderId<AppEvent>>,
}

impl<'a> System<'a> for EventProcessSystem {
    type SystemData = Read<'a, EventChannel<AppEvent>>;

    // Систему можно инициализировать до начала работы с помощью вызова setup
    fn setup(&mut self, world: &mut World) {
        println!("EventProcessSystem setup called");

        // Инициализируем системные данные сначала, создавая тем самым канал
        Self::SystemData::setup(world);

        // Затем мы можем получить канал событий, он является ресурсом, который будет здесь создан
        let mut channel = world.fetch_mut::<EventChannel<AppEvent>>();
        self.reader = Some(channel.register_reader());
    }

    fn run(&mut self, events: Self::SystemData) {
        // К моменту начала работы у нас должен быть уже канал чтения событий
        let reader = self.reader.as_mut().unwrap();
        for event in events.read(reader) {
            match event {
                AppEvent::Created => {
                    println!("Message received: Created")
                },
                AppEvent::Moved => {
                    println!("Message received: Moved")
                }
            }
        }
    }
}