use eyre::{Context, ContextCompat};
use log::{debug, info, LevelFilter};
use petgraph::{
    algo::{astar, dijkstra, min_spanning_tree},
    data::FromElements,
    dot::{Config, Dot},
    graph::{NodeIndex, UnGraph},
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////

fn execute_app() -> Result<(), eyre::Error> {
    // Создаем ненаправленный граф с узлами типа i32, а ребрами типа ()
    let g = UnGraph::<i32, ()>::from_edges(&[(1, 2), (2, 3), (3, 4), (1, 4), (1, 5), (5, 4)]);
    for neighbor in g.neighbors(1.into()){
        info!("Neighbor of node 1: {}", neighbor.index());
    }

    // Находим стоимость пути между узлами, используя стоимость 1 в качестве веса ребра
    // для всех ребер
    let node_map = dijkstra(&g, 1.into(), None, |_| 1); // Some(4.into())
    for node in node_map.iter() {
        info!("Dejkstra: weight from 1 to {} is {}", node.0.index(), node.1);
    }
    assert_eq!(&1_i32, node_map.get(&NodeIndex::new(4)).unwrap());
    assert_eq!(&2_i32, node_map.get(&NodeIndex::new(3)).unwrap());

    // Получаем минимальное остовное дерево из исходного графа. Затем проверяем,
    // что лишние узлы были обрезаны.
    // Остовное дерево значит, что это дерево содержит лишь ребра, по которым можно добраться
    // из одного места в другое минимальным путем, отбрасывая все лишние пути, которые только
    // увеличивают стоимость
    let mst = UnGraph::<_, _>::from_elements(min_spanning_tree(&g));
    assert_eq!(g.raw_edges().len() - 1, mst.raw_edges().len() + 1);

    // Ищем кратчайший путь из одного узла в другой
    // Алгоритм A*  (A со звездочкой)
    let shortest_path = astar(&g, 1.into(), |node| node.index() == 4, |_| 1, |_| 0).expect("Path found");
    info!("Shortest path: 1 to 4 is {:?}, weight {}", shortest_path.1, shortest_path.0);

    // Output the tree to `graphviz` `DOT` format
    println!("{:?}", Dot::with_config(&mst, &[Config::EdgeNoLabel]));
    // graph {
    //     0 [label="\"0\""]
    //     1 [label="\"0\""]
    //     2 [label="\"0\""]
    //     3 [label="\"0\""]
    //     1 -- 2
    //     3 -- 4
    //     2 -- 3
    // }

    Ok(())
}

fn main() {
    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Настройка логирования на основании количества флагов verbose
    setup_logging().expect("Logging setup");

    // Запуск приложения
    if let Err(err) = execute_app() {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
