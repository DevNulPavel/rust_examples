use std::os::raw::c_void;

trait Pet {
    fn sound(&self) -> String;
    fn name(&self) -> String;
}

struct Cat {
    _life: u8,
    _age: u8,
    name: String,
}

impl Cat {
    fn new(name: impl Into<String>) -> Self {
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

#[repr(C)]
#[derive(Copy, Clone)]
struct RawPetVtable {
    drop: usize,
    size: usize,
    align: usize,
    sound: usize,
    name: usize,
}

#[allow(dead_code)]
fn bark2() -> String {
    "Woof!".to_string()
}

fn add(a: usize, b: usize) -> String {
    format!("{} + {} = {}", a, b, a + b)
}

const POINTER_SIZE: usize = std::mem::size_of::<*const c_void>();

fn main() {
    unsafe {
        let mut kitty: Box<dyn Pet> = Box::new(Cat::new("Kitty"));

        let addr_of_data_ptr = &mut kitty as *mut _ as *mut c_void as usize;
        let addr_of_pointer_to_vtable = addr_of_data_ptr + POINTER_SIZE;
        let ptr_to_ptr_to_vtable = addr_of_pointer_to_vtable as *mut *const RawPetVtable;
        let mut new_vtable = **ptr_to_ptr_to_vtable;
        new_vtable.sound = add as *const c_void as usize;
        *ptr_to_ptr_to_vtable = &new_vtable;

        greet_pet(kitty);
    }
}

fn greet_pet(pet: Box<dyn Pet>) {
    println!("You: Hello, {}!", pet.name());
    println!("{}: {}\n", pet.name(), pet.sound());
}
