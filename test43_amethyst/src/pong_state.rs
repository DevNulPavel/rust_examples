use amethyst::{
    prelude::*,
    assets::{
        Handle, 
    },
    renderer::{
        SpriteSheet,
    }
};

pub struct PongState {
    _sprite_sheet_handle: Handle<SpriteSheet>
}

impl PongState{
    pub fn new(_sprite_sheet_handle: Handle<SpriteSheet>) -> Self {
        PongState{
            _sprite_sheet_handle
        }
    }
}

// Реализуем простое игровое состояние
impl SimpleState for PongState {
    // Старт игрового состояния
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Состояние менять не надо
        Trans::None
    }
}
