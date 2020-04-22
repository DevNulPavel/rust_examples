mod systems;
mod components;
mod resources;

use specs::{
    Builder, 
    DispatcherBuilder, 
    World, 
    WorldExt,
};
use crate::{
    systems::*,
    components::*,
    resources::*
};


fn main() {
    // Мир - это хранилище компонентов
    let mut world = World::new();

    // Регистрируем работу с компонентами
    world.register::<PositionComponent>();
    world.register::<VelocityComponent>();
    world.register::<StoneComponent>();
    world.register::<TargetComponent>();
    world.register::<DataComponent>();
    
    // Первая сущность имеет только компонент позиции
    world
        .create_entity()
        .with(PositionComponent { x: 4.0, y: 7.0 })
        .build();
    // Создаем сущность с компонентами позиции и ускорения
    world
        .create_entity()
        .with(PositionComponent { x: 2.0, y: 5.0 })
        .with(VelocityComponent { x: 0.1, y: 0.2 })
        .build();
    
    // Добавить ресурс можно следующим убразом
    world.insert(DeltaTime::new(0.05));
    // Обновить ресурс можно так
    {
        let mut delta = world.write_resource::<DeltaTime>();
        *delta = DeltaTime::new(0.04);
    }

    // Создаем новый диспетчер, который содержит в себе логику систем
    let mut dispatcher = DispatcherBuilder::new()
        .with(StoneCreatorSystem::default(), "stone_creator", &[])
        .with(DataModifiedProcSystem::default(), "data_modified", &[])
        .with_barrier()
        .with(HelloWorldSystem, "hello_world", &["stone_creator"])
        .with(UpdatePosSystem, "update_pos", &["hello_world"])
        .with(HelloWorldSystem, "hello_updated", &["update_pos"])
        .with(EventProcessSystem::default(), "event_process", &["stone_creator", "update_pos"])
        .with(FollowTargetSystem, "follow_target", &["update_pos"])
        // Данная система будет выполняться всегда самой последней + в главном одном потоке?
        // Она не может иметь каких-то зависимостей
        .with_thread_local(RenderingSystem)
        .build();

    // Вызывает setup у всех систем в порядке как создано дерево
    dispatcher.setup(&mut world);
    
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Выполняем наши системы в мире
        dispatcher.dispatch(&world);

        // Поддерживаем сущности в активном состоянии, а так же удаляем те, которые помечены на удаление
        world.maintain();

        println!();
    }
}