use std::{
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    hash::{Hash, Hasher},
};
// use {};

////////////////////////////////////////////////////////////////////////////////////////////////

/// Базовый нод в дереве
#[derive(PartialEq, Eq, Debug)]
pub enum Node {
    /// Отдельный лист
    Leaf { value: &'static str },

    /// Ветка с нодами
    Branch { children: Vec<Node> },
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Ссылка на нод в дереве с индексом
#[derive(Clone, Copy, Debug)]
struct TreeNodeRef<'a> {
    /// Индекс
    tree_index: usize,

    /// Ссылка на ноду
    node: &'a Node,
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Различные операции над деревом
enum Op<'a, I> {
    /// Обход дерева с помощью указанного итератора
    TraverseForest { trees_iter: I },

    /// Проверить ссылку на ноду
    AnalyzeNode { tree_node_ref: TreeNodeRef<'a> },

    /// Обойти ветку
    TraverseBranch {
        /// Ссылка на ноду с индексом
        tree_node_ref: TreeNodeRef<'a>,

        /// Итератор по чилдам
        children_iter: I,

        /// Хешер стандартный
        hasher: DefaultHasher,

        /// Текущий размер ветки
        current_size: usize,
    },

    /// Выход из ветки?
    Return {
        /// Ссылка на ноду
        tree_node_ref: TreeNodeRef<'a>,

        /// Хеш от ноды
        hash: u64,

        /// Размер
        size: usize,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Поддерево
#[derive(Debug)]
struct SubtreeNodeRef<'a> {
    /// Индекс
    expected_tree_index: usize,

    /// Корень поддерева
    subtree_root: &'a Node,

    /// Размер поддерева
    subtree_size: usize,
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Поиск наибольшего поддерева
pub fn largest_common_subtree(forest: &[Node]) -> Option<&Node> {
    // Массив операций.
    let mut ops = vec![Op::TraverseForest {
        // Начинаем с обхода дерева, добавляя итератор на корень в качестве инициализации.
        // Итерация начнется с первого элемента.
        // Итератор делаем с поддержкой перечисления
        trees_iter: forest.iter().enumerate(),
    }];

    // Хешмапа с поддеревьями с помощью которой мы будем
    // как раз находить наличие поддеревьев
    let mut subtrees: HashMap<u64, SubtreeNodeRef<'_>> = HashMap::new();

    // Главный цикл обработки
    loop {
        // Извлекаем последнюю операцию в массиве операций
        let Some(op) = ops.pop() else {
            // Операции закончились - выход
            break;
        };

        // Если есть операция, смотрим что за операция
        match op {
            // Обход дерева с помощью определенного итератора по нодам
            Op::TraverseForest { mut trees_iter } => {
                // Из итератора берем очередной элемент следующий лишь раз + индекс.
                // Если очередного элемента здесь нету, то это значит, что больше нету нодов на текущем уровне.
                if let Some((tree_index, tree)) = trees_iter.next() {
                    // Добавляем в операции очередную следующую итерацию текущего итератора
                    ops.push(Op::TraverseForest { trees_iter });

                    // Добавляем операцию проверки ноды текущей из итератора
                    ops.push(Op::AnalyzeNode {
                        tree_node_ref: TreeNodeRef {
                            tree_index,
                            node: tree,
                        },
                    });
                }
            }

            // Непосредственно проверка конкретной ноды
            Op::AnalyzeNode {
                tree_node_ref,
            } => {
                // Смотрим на тип нашей ноды
                match &tree_node_ref.node {
                    // Тип ноды - лист
                    Node::Leaf { value } => {
                        // Считаем хеш от значения в листе.
                        let hash = {
                            // Создаем стандартный хешер c нулевыми ключами инициализации.
                            let mut hasher = DefaultHasher::new();

                            // Хешируем значение листа в хешер
                            value.hash(&mut hasher);

                            hasher.finish()
                        };

                        // Добавляем новую операцию возврата, сохраняя туда хеш и текущий нод с индексом
                        ops.push(Op::Return {
                            tree_node_ref,
                            hash,
                            size: 1,
                        });
                    }

                    // Тип ноды - нода с детьми
                    Node::Branch { children } => {
                        // Создаем итератор по всем детям данного узла
                        let children_iter = children.iter().enumerate();

                        // Создаем хешер, который будет накапливать хеши от листов и от индексов в дереве
                        let hasher = DefaultHasher::new();

                        // Добавляем новую операцию - обход детей ветки
                        ops.push(Op::TraverseBranch {
                            tree_node_ref,
                            children_iter,
                            hasher,
                            current_size: 1,
                        });
                    }
                }
            }

            // Обход узла, котоый содержит детей (ветка)
            Op::TraverseBranch {
                tree_node_ref,
                mut children_iter,
                mut hasher,
                current_size,
            } => {
                // Извлекаем из итератора детей очередной элемент
                match children_iter.next() {
                    // Больше детей нету никаких
                    None => {
                        // Заканчиваем расчет хеша ветки чилдов
                        let hash = hasher.finish();

                        // Можно добавить операцию возврата.
                        ops.push(Op::Return {
                            tree_node_ref,
                            hash,
                            size: current_size,
                        });
                    }

                    // Еще есть чилды какие-то
                    Some((child_index, child_node)) => {
                        // Хешируем индекс в общий хешер.
                        // Это нужно для того, чтобы проверять, что поддерево действительно одинаковое и не содержит
                        // лишних узлов на пути.
                        child_index.hash(&mut hasher);

                        // Добавляем операцию очередной итерации проверки чилда,
                        // записывая туда текущий незавершенный итератор.
                        ops.push(Op::TraverseBranch {
                            tree_node_ref,
                            children_iter,
                            hasher,
                            current_size,
                        });

                        // Сохраняем узел для проверки
                        ops.push(Op::AnalyzeNode {
                            tree_node_ref: TreeNodeRef {
                                node: child_node,
                                ..tree_node_ref
                            },
                        });
                    }
                }
            }

            // Операция возврата
            Op::Return {
                tree_node_ref,
                hash,
                size,
            } => {
                // Смотрим последнюю операцию в очереди
                match ops.last_mut().unwrap() {
                    // Обход дерева - ничего не делает
                    Op::TraverseForest { .. } => {}

                    // Обход ветки, можно там в последнем элементе обновить значения
                    Op::TraverseBranch {
                        hasher,
                        current_size,
                        ..
                    } => {
                        // TODO: ???
                        // Хешируем имеющийся хеш от узла снова в хешер обхода ветки
                        hash.hash(hasher);

                        // Текущий размер в последнем элементе увеличиваем на указанное значение.
                        *current_size += size;
                    }

                    _ => {
                        panic!("Other operation cannot posible");
                    }
                }

                // Проверяем наличие уникального поддерева с таким же хешем.
                // Хеш считается суммарно от значений в листах и от значений индексов веток в поддереве.
                // Тем самым достигается проверка уникальности порядка в поддереве.
                match subtrees.entry(hash) {
                    // Такого поддерева там нету + глубина у нас нулевая
                    Entry::Vacant(ev) if tree_node_ref.tree_index == 0 => {
                        // Значит сам узел и является общим поддеревом у всего дерева
                        ev.insert(SubtreeNodeRef {
                            expected_tree_index: 1,
                            subtree_root: tree_node_ref.node,
                            subtree_size: size,
                        });
                    }

                    // Такого поддерева не было сохранено в хешах, значит его не было
                    Entry::Vacant(..) => (),

                    // Такое поддерево есть, но текущее дерево имеет нулевой индекс
                    Entry::Occupied(..) if tree_node_ref.tree_index == 0 => (),

                    // Такое поддерево есть и индекс совпадает
                    Entry::Occupied(mut eo)
                        if tree_node_ref.tree_index == eo.get().expected_tree_index =>
                    {
                        // Берем поддерево мутабельно
                        let subtree_entry = eo.get_mut();
                        // Увеличиваем индекс поддерева
                        subtree_entry.expected_tree_index += 1;
                    }
                    // Все остальные поддеревья
                    Entry::Occupied(..) => (),
                }
            }
        }
    }

    // Перегоняем хешмапу с поддеревьями
    subtrees
        .into_values()
        // Фильтруем лишь ожидаемую длину дерева
        .filter(|entry| entry.expected_tree_index == forest.len())
        // Находим поддерево максимального размера
        .max_by_key(|entry| entry.subtree_size)
        // Возвращаем корень поддерева
        .map(|entry| entry.subtree_root)
}
