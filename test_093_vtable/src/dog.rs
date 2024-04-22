use crate::pet::Pet;

////////////////////////////////////////////////////////////////////////////////

pub(super) struct Dog {
    _age: u8,
    name: String,
}

impl Dog {
    pub(super) fn new(name: impl Into<String>) -> Self {
        Self {
            _age: 0,
            name: name.into(),
        }
    }
}

impl Pet for Dog {
    fn sound(&self) -> String {
        "Woof!".to_string()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
