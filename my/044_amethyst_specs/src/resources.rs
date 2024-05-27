
#[derive(Default)]
pub struct DeltaTime{
    pub time: f32
}

impl DeltaTime{
    pub fn new(time: f32) -> Self{
        DeltaTime{
            time
        }
    }
}