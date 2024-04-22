use std::fmt::Write;

////////////////////////////////////////////////////////////////////////////////////////////////

/// Сначала устанавливаем непосредственно сам наш аллокатор
#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

////////////////////////////////////////////////////////////////////////////////////////////////

/// Через специальную переменную мы можем указать конфиг для jemalloc.
///
/// Без флага сборки `unprefixed_malloc_on_supported_platforms`
/// переменная должна называться `_rjem_malloc_conf`.
///
/// Ссылки разные на конфиг:
/// - https://docs.rs/tikv-jemalloc-sys/0.5.4+5.3.0-patched/tikv_jemalloc_sys/static.malloc_conf.html
/// - https://github.com/jemalloc/jemalloc/blob/dev/INSTALL.md
/// - https://github.com/jemalloc/jemalloc/blob/dev/TUNING.md
/// - https://jemalloc.net/jemalloc.3.html#tuning
///
/// Параметры:
/// - `prof:true` включает поддержку профилирования
/// - `prof_active:false` не активирует само профилирование сразу же со старта
/// - `lg_prof_sample:19` говорит, что период семплирования аллокаций 2^19 байт = 512KiB
#[cfg(target_os = "linux")]
#[allow(non_upper_case_globals)]
#[export_name = "malloc_conf"]
pub static malloc_conf: &[u8] = b"prof:true,prof_active:false,lg_prof_sample:19,prof_leak:false\0";

////////////////////////////////////////////////////////////////////////////////////////////////

fn test_alloc_1() -> Vec<u64> {
    // Создаем вектор без аллокаций
    let mut buffer = Vec::<u64>::new();

    // Заполняем этот самый буффер разной всякой фигней
    for i in 0..(1024 * 1024 * 10) {
        buffer.push(i);
    }

    buffer
}

fn test_alloc_2() -> String {
    let mut buffer = String::with_capacity(1024 * 1024 * 4);

    // Заполняем этот самый буффер разной всякой фигней
    for _ in 0..(1024 * 1024 * 40) {
        buffer.write_str("test").unwrap();
    }

    buffer
}

////////////////////////////////////////////////////////////////////////////////////////////////

fn enable_pprof() {
    #[cfg(target_os = "linux")]
    {
        // Получаем из глобального инстанса контроллера профилирования
        // непосредственно блокировку.
        let mut prof = jemalloc_pprof::PROF_CTL.as_ref().unwrap().blocking_lock();

        // Вызываем активацию
        prof.activate().unwrap();
    }

    // Включаем профилирование в нужный момент.
    // Можно так же это делать через инстанс контроллера.
    // #[cfg(target_os = "linux")]
    // jemalloc_pprof::activate_jemalloc_profiling().await;
}

fn disable_pprof() {
    #[cfg(target_os = "linux")]
    {
        // Получаем из глобального инстанса контроллера профилирования
        // непосредственно блокировку.
        let mut prof = jemalloc_pprof::PROF_CTL.as_ref().unwrap().blocking_lock();

        prof.deactivate().unwrap();
    }

    // Выключаем профилирование в нужный момент.
    // Можно так же это делать через инстанс контроллера.
    // #[cfg(target_os = "linux")]
    // jemalloc_pprof::deactivate_jemalloc_profiling();
}

fn profile() {
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
        std::fs::write("./output/heap.pb.gz", pprof_data).unwrap();

        println!("Memory profile info has written");
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    enable_pprof();

    // Аллокации
    let _v1 = test_alloc_1();

    // Аллокации
    let _v2 = test_alloc_2();

    profile();

    disable_pprof();
}
