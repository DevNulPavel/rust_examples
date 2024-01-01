/// Сначала устанавливаем непосредственно сам наш аллокатор
#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Через специальную переменную мы можем указать конфиг для jemalloc.
///
/// Без флага сборки `unprefixed_malloc_on_supported_platforms`
/// переменная должна называться `_rjem_malloc_conf`.
///
/// Ссылки разные:
/// -
///
/// Параметры:
/// - `prof:true` включает поддержку профилирования
/// - `prof_active:false` не активирует само профилирование сразу же со старта
/// - `lg_prof_sample:19` говорит, что период семплирования аллокаций 2^19 байт = 512KiB
#[cfg(target_os = "linux")]
#[allow(non_upper_case_globals)]
#[export_name = "malloc_conf"]
pub static malloc_conf: &[u8] = b"prof:true,prof_active:false,lg_prof_sample:19\0";

fn main() {
    // Создаем вектор без аллокаций
    let mut buffer = Vec::<u64>::new();

    // Включаем профилирование в нужный момент.
    // Можно так же это делать через инстанс контроллера.
    #[cfg(target_os = "linux")]
    jemalloc_pprof::activate_jemalloc_profiling();

    // Заполняем этот самый буффер разной всякой фигней
    for i in 0..(1024 * 10) {
        buffer.push(i);
    }

    #[cfg(target_os = "linux")]
    {
        // Получаем из глобального инстанса контроллера профилирования
        // непосредственно блокировку.
        let mut prof = jemalloc_pprof::PROF_CTL.as_ref().unwrap().blocking_lock();

        // Делаем проверку, что сейчас профилирование вообще активно
        assert!(prof.activated());

        // 2^19 должна быть частота семплирования
        assert_eq!(prof.lg_sample(), 19);

        // Выводим информацию по профилированию
        println!("Profiling metadata: {:?}", prof.get_md());

        // Делаем дамп во временный файлик с помощью команды `prof.dump`, затем конвертируем
        // эти данные из временного файлика в формат pprof, возвращая сконвертированные данные.
        let pprof_data = prof.dump_pprof().unwrap();

        // Теперь можно уже записать эти данные в файлик нужный
        std::fs::write("heap.pb.gz", pprof_data).unwrap();
    }

    // Выключаем профилирование в нужный момент.
    // Можно так же это делать через инстанс контроллера.
    #[cfg(target_os = "linux")]
    jemalloc_pprof::deactivate_jemalloc_profiling();
}
