use super::calc_result::CalcResult;

// Содержимое нашего сообщения
#[derive(Default)]
pub struct ValuesMessage{
    pub x: i32, 
    pub y: i32
}

impl ValuesMessage{
    pub fn new(x: i32, y: i32)-> ValuesMessage{
        ValuesMessage{ x, y }
    }
}

// Реализация трейта Message для нашего сообщения
impl actix::Message for ValuesMessage {
    // описываем тип возвращаемого значения на сообщение
    //type Result = Option<CalcResult>;
    type Result = CalcResult;
}
