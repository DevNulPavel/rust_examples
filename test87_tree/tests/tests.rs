use test87_tree::{largest_common_subtree, Node};

#[test]
fn empty_forest() {
    assert_eq!(largest_common_subtree(&[]), None);
}

#[test]
fn one_leaf_tree_forest() {
    assert_eq!(
        largest_common_subtree(&[Node::Leaf { value: "alpha" },]),
        Some(&Node::Leaf { value: "alpha" }),
    );
}

#[test]
fn one_empty_branch_tree_forest() {
    assert_eq!(
        largest_common_subtree(&[Node::Branch { children: vec![] },]),
        Some(&Node::Branch { children: vec![] }),
    );
}

#[test]
fn two_leaf_trees_forest_with_no_common_subtrees() {
    assert_eq!(
        largest_common_subtree(&[Node::Leaf { value: "alpha" }, Node::Leaf { value: "beta" },]),
        None,
    );
}

#[test]
fn two_equal_leaf_trees_forest() {
    assert_eq!(
        largest_common_subtree(&[Node::Leaf { value: "alpha" }, Node::Leaf { value: "alpha" },]),
        Some(&Node::Leaf { value: "alpha" }),
    );
}

#[test]
fn two_equal_empty_branches_forest() {
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
                                },],
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
