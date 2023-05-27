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
struct TreeNode<'a> {
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
    AnalyzeNode { tree_node: TreeNode<'a> },

    /// Обойти ветку
    TraverseBranch {
        /// Ссылка на ноду с индексом
        tree_node: TreeNode<'a>,

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
        tree_node: TreeNode<'a>,

        /// Хеш от ноды
        hash: u64,

        /// Размер
        size: usize,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Поддерево
#[derive(Debug)]
struct SubtreeEntry<'a> {
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

    // Хешмапа с поддеревьями
    let mut subtrees: HashMap<u64, SubtreeEntry<'_>> = HashMap::new();

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
                        tree_node: TreeNode {
                            tree_index,
                            node: tree,
                        },
                    });
                }
            }

            // Непосредственно проверка конкретной ноды
            Op::AnalyzeNode { tree_node } => {
                // Смотрим на тип нашей ноды
                match &tree_node.node {
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
                            tree_node,
                            hash,
                            size: 1,
                        });
                    }

                    // Тип ноды - нода с детьми
                    Node::Branch { children } => {
                        // Создаем итератор по всем детям данного узла
                        let children_iter = children.iter().enumerate();

                        // Создаем хешер, который будет накапливать хеши
                        let hasher = DefaultHasher::new();

                        // Добавляем новую операцию - обход детей ветки
                        ops.push(Op::TraverseBranch {
                            tree_node,
                            children_iter,
                            hasher,
                            current_size: 1,
                        });
                    }
                }
            }

            // Обход узла, котоый содержит детей (ветка)
            Op::TraverseBranch {
                tree_node,
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
                            tree_node,
                            hash,
                            size: current_size,
                        });
                    }

                    // Еще есть чилды какие-то
                    Some((child_index, child_node)) => {
                        // Хешируем индекс хеша в общий хешер
                        child_index.hash(&mut hasher);

                        // Добавляем операцию очередной итерации проверки чилда,
                        // записывая туда текущий незавершенный итератор.
                        ops.push(Op::TraverseBranch {
                            tree_node,
                            children_iter,
                            hasher,
                            current_size,
                        });

                        // Сохраняем узел для проверки
                        ops.push(Op::AnalyzeNode {
                            tree_node: TreeNode {
                                node: child_node,
                                ..tree_node
                            },
                        });
                    }
                }
            }

            // Операция возврата
            Op::Return {
                tree_node,
                hash,
                size,
            } => {

                // Смотрим последнюю операцию в очереди
                match ops.last_mut().unwrap() {
                    // Обход дерева - ничего не делает
                    Op::TraverseForest { .. } => {

                    },
                    // Обход ветки, можно там в последнем элементе обновить значения
                    Op::TraverseBranch {
                        hasher,
                        current_size,
                        ..
                    } => {
                        // TODO: ???
                        // Хешируем имеющийся хеш снова в хешер?
                        hash.hash(hasher);

                        // Текущий размер в последнем элементе увеличиваем на указанное значение.
                        *current_size += size;
                    }
                    _ => {
                        panic!("Other operation cannot posible");
                    }
                }

                match subtrees.entry(hash) {
                    Entry::Vacant(ev) if tree_node.tree_index == 0 => {
                        ev.insert(SubtreeEntry {
                            expected_tree_index: 1,
                            subtree_root: tree_node.node,
                            subtree_size: size,
                        });
                    }
                    Entry::Vacant(..) => (),
                    Entry::Occupied(..) if tree_node.tree_index == 0 => (),
                    Entry::Occupied(mut eo)
                        if tree_node.tree_index == eo.get().expected_tree_index =>
                    {
                        let subtree_entry = eo.get_mut();
                        subtree_entry.expected_tree_index += 1;
                    }
                    Entry::Occupied(..) => (),
                }
            }
        }
    }

    subtrees
        .into_values()
        .filter(|entry| entry.expected_tree_index == forest.len())
        .max_by_key(|entry| entry.subtree_size)
        .map(|entry| entry.subtree_root)
}
