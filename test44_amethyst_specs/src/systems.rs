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

// Можно определить собственный класс системных данных
#[derive(SystemData)]
pub struct UpdatePosSystemData<'a> {
    time      : Read<'a, DeltaTime>,
    entities  : Entities<'a>,
    velocity  : ReadStorage<'a, VelocityComponent>,
    position  : WriteStorage<'a, PositionComponent>,
    stones    : ReadStorage<'a, StoneComponent>,
    events    : Write<'a, EventChannel<AppEvent>>
}

pub struct UpdatePosSystem;

impl<'a> System<'a> for UpdatePosSystem {
    type SystemData = UpdatePosSystemData<'a>;
    
    fn run(&mut self, mut data: Self::SystemData) {
        // Находим все сущности, которые содержат и ускорение, и позицию
        // Мы можем обернуть 
        // Если надо исключить какие-то компоненты, тогда можно добавить ! перед & - "!&data.velocity"
        for (e, vel, pos, stone) in (&data.entities, &data.velocity, &mut data.position, (&data.stones).maybe()).join() {
            pos.x += vel.x * data.time.time;
            pos.y += vel.y * data.time.time;

            // Так мы можем проверить, содержит ли данная сущность компонент камня
            if stone.is_some() {
                println!("This is stone");
                // Отправка сообщения в канал
                data.events.single_write(AppEvent::StoneMoved(e));
            }
        }
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
        data.events_channel.single_write(AppEvent::StoneCreated(stone));
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AppEvent{
    StoneCreated(Entity),
    StoneMoved(Entity),
}

#[derive(Default)]
pub struct EventProcessSystem {
    reader: Option<ReaderId<AppEvent>>,
}

impl<'a> System<'a> for EventProcessSystem {
    type SystemData = (Read<'a, EventChannel<AppEvent>>,
                       ReadStorage<'a, StoneComponent>);

    // Систему можно инициализировать до начала работы с помощью вызова setup
    fn setup(&mut self, world: &mut World) {
        println!("EventProcessSystem setup called");

        // Инициализируем системные данные сначала, создавая тем самым канал
        Self::SystemData::setup(world);

        // Затем мы можем получить канал событий, он является ресурсом, который будет здесь создан
        let mut channel = world.fetch_mut::<EventChannel<AppEvent>>();
        self.reader = Some(channel.register_reader());
    }

    fn run(&mut self, (events, stones): Self::SystemData) {
        // К моменту начала работы у нас должен быть уже канал чтения событий
        let reader = self.reader.as_mut().unwrap();
        for event in events.read(reader) {
            match event {
                AppEvent::StoneCreated(_) => {
                    println!("Message received: Created");
                },
                AppEvent::StoneMoved(entity) => {
                    println!("Message received: Moved");

                    // Получили сущность, получаем для нее компонент
                    let stone_component: Option<&StoneComponent> = stones.get(*entity);
                    if stone_component.is_some() {
                        println!("Message received: Stone component received")
                    }
                }
            }
        }
    }
}