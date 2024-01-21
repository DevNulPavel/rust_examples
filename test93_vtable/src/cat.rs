use crate::pet::Pet;

////////////////////////////////////////////////////////////////////////////////

pub(super) struct Cat {
    _life: u8,
    _age: u8,
    name: String,
}

impl Cat {
    pub(super) fn new(name: impl Into<String>) -> Self {
        Self {
            _life: 9,
            _age: 0,
            name: name.into(),
        }
    }
}

impl Pet for Cat {
    fn sound(&self) -> String {
        "Meow!".to_string()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
