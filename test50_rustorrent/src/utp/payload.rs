
pub(super) const PAYLOAD_SIZE: usize = 1500;

#[repr(C, packed)]
pub(super) struct Payload {
    /// Массив с данными
    pub data: [u8; PAYLOAD_SIZE],
    /// Полезных данных
    pub len: usize
}

impl Payload {
    /// Размещаем данные в объекте Payload
    pub(super) fn new_in_place(place: &mut Payload, data: &[u8]) {
        let data_len = data.len();
        place.data[..data_len].copy_from_slice(data);
        place.len = data_len;
    }

    /// Создаем новые данные из буффера
    pub(super) fn new(data: &[u8]) -> Payload {
        let data_len = data.len();
        let mut payload = [0; PAYLOAD_SIZE];
        payload[..data_len].copy_from_slice(data);
        Payload {
            data: payload,
            len: data_len
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
