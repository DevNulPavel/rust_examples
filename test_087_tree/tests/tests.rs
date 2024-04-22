use test87_tree::{largest_common_subtree, Node};

#[test]
fn empty_forest() {
    // Для пустого дерева у нас нету общего поддерева
    assert_eq!(largest_common_subtree(&[]), None);
}

#[test]
fn one_leaf_tree_forest() {
    // Для дерева из одного листа у нас общим поддеревом является лишь текущий лист
    assert_eq!(
        largest_common_subtree(&[Node::Leaf { value: "alpha" },]),
        Some(&Node::Leaf { value: "alpha" }),
    );
}

#[test]
fn one_empty_branch_tree_forest() {
    // Если дерево состояит лишь из одной пустой ветки,
    // то общей будет эта самая пустая ветка.
    assert_eq!(
        largest_common_subtree(&[Node::Branch { children: vec![] },]),
        Some(&Node::Branch { children: vec![] }),
    );
}

#[test]
fn two_leaf_trees_forest_with_no_common_subtrees() {
    // Если в дереве все узлы уникальные, тогда там просто нету общего поддерева
    assert_eq!(
        largest_common_subtree(&[Node::Leaf { value: "alpha" }, Node::Leaf { value: "beta" },]),
        None,
    );
}

#[test]
fn two_equal_leaf_trees_forest() {
    // Если в дереве у нас 2 узла одинаковых, тогда общим будет этот самый угол
    assert_eq!(
        largest_common_subtree(&[Node::Leaf { value: "alpha" }, Node::Leaf { value: "alpha" },]),
        Some(&Node::Leaf { value: "alpha" }),
    );
}

#[test]
fn two_equal_empty_branches_forest() {
    // Если в дереве у нас 2 ветки, но там пустые дети, тогда общей будет эта самая ветка
    assert_eq!(
        largest_common_subtree(&[
            Node::Branch { children: vec![] },
            Node::Branch { children: vec![] },
        ]),
        Some(&Node::Branch { children: vec![] }),
    );
}

#[test]
fn three_various_trees_forest_with_common_subtree() {
    // Полноценный поиск поддерева, совпадающим с узлами дочерними
    assert_eq!(
        largest_common_subtree(&[
            Node::Branch {
                children: vec![
                    Node::Leaf { value: "alpha" },
                    Node::Branch {
                        children: vec![Node::Leaf { value: "beta" },],
                    },
                ],
            },
            Node::Branch {
                children: vec![
                    Node::Leaf { value: "gamma" },
                    Node::Branch {
                        children: vec![
                            Node::Leaf { value: "alpha" },
                            Node::Branch {
                                children: vec![Node::Leaf { value: "beta" },],
                            },
                        ],
                    },
                    Node::Leaf { value: "delta" },
                ],
            },
            Node::Branch {
                children: vec![
                    Node::Branch {
                        children: vec![
                            Node::Leaf { value: "gamma" },
                            Node::Branch {
                                children: vec![
                                    Node::Leaf { value: "alpha" },
                                    Node::Branch {
                                        children: vec![Node::Leaf { value: "beta" },],
                                    },
                                ],
                            },
                        ],
                    },
                    Node::Branch {
                        children: vec![Node::Leaf { value: "delta" },],
                    },
                ],
            },
        ]),
        // Общим поддеревом будет как раз общая ветка
        Some(&Node::Branch {
            children: vec![
                Node::Leaf { value: "alpha" },
                Node::Branch {
                    children: vec![Node::Leaf { value: "beta" },],
                },
            ],
        }),
    );
}

#[test]
fn three_various_trees_forest_with_no_common_subtree() {
    // Две ветки совпадают как поддерево, а третья нет
    assert_eq!(
        largest_common_subtree(&[
            Node::Branch {
                children: vec![
                    Node::Leaf { value: "alpha" },
                    Node::Branch {
                        children: vec![Node::Leaf { value: "beta" },],
                    },
                ],
            },
            Node::Branch {
                children: vec![
                    Node::Leaf { value: "gamma" },
                    Node::Branch {
                        children: vec![
                            Node::Leaf { value: "alpha" },
                            Node::Branch {
                                children: vec![Node::Leaf { value: "beta" },],
                            },
                        ],
                    },
                    Node::Leaf { value: "delta" },
                ],
            },
            Node::Branch {
                children: vec![
                    Node::Branch {
                        children: vec![
                            Node::Leaf { value: "gamma" },
                            Node::Branch {
                                children: vec![Node::Branch {
                                    children: vec![Node::Leaf { value: "epsilon" },],
                                }],
                            },
                        ],
                    },
                    Node::Branch {
                        children: vec![Node::Leaf { value: "delta" },],
                    },
                ],
            },
        ]),
        None,
    );
}
