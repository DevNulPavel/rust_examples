use mem_hog::*;
use std::io::BufRead;

#[cfg(feature = "jemallocator")]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// #[cfg(feature = "tikv-jemallocator")]
// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Gets the input from stdin.
pub fn get_input() -> Result<u32, String> {
    // TODO: Замена на smallstr
    let mut buf = String::with_capacity(8);

    match std::io::stdin().lock().read_line(&mut buf) {
        Ok(_) => {}
        Err(e) => panic!("Error reading the input: {e}"),
    }

    buf.trim()
        .parse()
        .map_err(|e| format!("Couldn't parse the input: {e:?}"))
}

/// Print the available commands.
fn print_commands() {
    use std::io::Write;
    print!(
        "
Available Commands:
    1) Accumulate (1st technique).
    2) Accumulate (2nd technique).
    3) Accumulate (3rd technique).
    4) Perform a `malloc_trim(0)`.
    5) Clear the accumulator.
    6) Reset the accumulator.
    7) Change the insertion amount.
    0) Exit.
Choice: "
    );
    std::io::stdout().flush().unwrap();
}

fn main() {
    // Размер накопителя
    let mut accumulator_size = 0;

    // Размер
    let mut amount = 5_000_000;
    // let mut amount = 1_000_000;
    // let mut amount = 500_000;
    // let mut amount = 200_000;
    // let mut amount = 100_000;
    // let mut amount = 50_000;

    // Хешмапа для накопления
    let mut accumulator = HashMap::with_capacity(cast::usize(amount));

    loop {
        // Текущий размер накопителя
        print!("Accumulator Size = {accumulator_size}");

        // Выводим команды
        print_commands();

        // Получаем ввод
        let input = get_input();

        // Время старта выполнения
        let start_time = std::time::Instant::now();

        match input {
            Ok(1) => {
                fill_map_light(&mut accumulator, amount);
                accumulator_size += amount;
            }
            Ok(2) => {
                fill_map_iter(&mut accumulator, amount);
                accumulator_size += amount;
            }
            Ok(3) => {
                fill_map(&mut accumulator, amount);
                accumulator_size += amount;
            }
            Ok(4) => {
                #[cfg(target_os = "linux")]
                unsafe {
                    if libc::malloc_trim(0) == 1 {
                        println!("Memory released");
                    } else {
                        println!("No memory released");
                    }
                }
            }
            Ok(5) => {
                // Чистим значения в хемапе, но память остается для будущего использования
                accumulator.clear(); // Note that `clear` keeps the used memory allocated for future use.
                accumulator_size = 0;
            }
            Ok(6) => {
                // Полностью пересоздаем накопитель
                accumulator = HashMap::with_capacity(cast::usize(amount));
                accumulator_size = 0;
            }
            Ok(7) => match get_input() {
                Ok(x) => amount = x,
                Err(e) => eprintln!("{e}"),
            },
            Ok(0) => {
                // don't drop anything
                std::process::exit(0);
                // return
            }
            Ok(x) => eprintln!("Invalid input: {x}"),
            Err(e) => eprintln!("{e}"),
        }
        println!("Executed in {:?}\n", std::time::Instant::now() - start_time);
    }
}
