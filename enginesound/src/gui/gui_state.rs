
use crate::gui::{
    WATERFALL_HEIGHT,
    WATERFALL_WIDTH
};

const WATERFALL_BUFFER_SIZE: usize = (WATERFALL_WIDTH * WATERFALL_HEIGHT) as usize;

/// Contains the waterfall bitmap
pub struct GUIState {    
    // Канал входных данных спектра
    input: crossbeam::Receiver<Vec<f32>>,

    // Буффер данных для отрисовки спектра
    pub(super) waterfall: [f32; WATERFALL_BUFFER_SIZE] // Расширяем область видимости только до родителя
}

impl GUIState {
    // На основании канала создаем новое состояние
    pub(crate) fn new(input: crossbeam::Receiver<Vec<f32>>) -> Self {
        // Сохраняем канал + создаем буффер нужного размера
        GUIState {
            input,
            waterfall: [0.07_f32; WATERFALL_BUFFER_SIZE],
        }
    }

    // Расширяем область видимости только до родителя
    pub(super) fn update(&mut self) {
        // Пока можем получать данные из канала
        while let Ok(new_line) = self.input.try_recv() {
            // Создаем итератор по всей ширине спектра
            let range_iter = 0..WATERFALL_WIDTH as usize; 

            // Создаем буффер значений для отрисовки лограрифмического спектра
            let log_scale = range_iter
                .into_iter()
                .map(|i| {
                    let new = (1.0 - (i + 1) as f32 / WATERFALL_WIDTH as f32).log2()
                        / (WATERFALL_WIDTH as f32).recip().log2()
                        * (WATERFALL_WIDTH - 1) as f32;
                        
                    let index1 = (new.floor() as usize).saturating_sub(1);
                    let index2 = new.floor() as usize;
                    let fract = new.fract();
                    let inv_fract = 1.0 - fract;

                    let val1  = new_line[index1];
                    let val2 = new_line[index2];

                    val1 * inv_fract + val2 * fract
                })
                .collect::<Vec<f32>>();

            self.add_line(&log_scale);
        }
    }

    /*// TODO: Переделать на итератор в качестве параметра
    // Смещает спектр вниз и рисует новые значения спектра
    fn add_line<I>(&mut self, line_iter: I)
    where
        I: std::iter::Iterator<Item=f32>
    {
        // Диапазон данных c начала до предпоследней строки
        let range_src = 0..((WATERFALL_WIDTH * (WATERFALL_HEIGHT - 1)) as usize);

        // Копируем данные в позицию после первой строки
        let copy_dst_pos = WATERFALL_WIDTH as usize;

        // Смещаем данные на строку вниз
        self.waterfall.copy_within(range_src,copy_dst_pos);

        // Записываем новую первую строку
        self.waterfall[0..WATERFALL_WIDTH as usize].copy_from_slice(line);
    }*/

    // Смещает спектр вниз и рисует новые значения спектра
    fn add_line(&mut self, line: &[f32]) {
        // Проверяем, что данные пришли правильного размера
        assert_eq!(
            line.len(),
            WATERFALL_WIDTH as usize,
            "wrong waterfall line width"
        );

        // Диапазон данных c начала до предпоследней строки
        let range_src = 0..((WATERFALL_WIDTH * (WATERFALL_HEIGHT - 1)) as usize);

        // Копируем данные в позицию после первой строки
        let copy_dst_pos = WATERFALL_WIDTH as usize;

        // Смещаем данные на строку вниз
        self.waterfall.copy_within(range_src,copy_dst_pos);

        // Записываем новую первую строку
        self.waterfall[0..WATERFALL_WIDTH as usize].copy_from_slice(line);
    }
}
