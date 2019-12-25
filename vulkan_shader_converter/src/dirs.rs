// Так можно отключать стандартную библиотеку
//#[no_std]; - в main
//#[no_implicit_prelude]; - в остальных

// Сокращаем пути
use std::path;
//use std::io;
use std::fs;

#[link(name = "dirs", vers = "1.0")]
pub fn visit_dirs(src_dir: &path::Path) -> Vec<std::path::PathBuf> {
    use std::collections::VecDeque;
    use std::path::PathBuf;
    //use std::vec::Vec; // Можно не указывать, так как итак в области видимости есть

    let mut paths: Vec<PathBuf> = Vec::new();

    // Создаем очередь, начинаем обработку с корневой папки
    let mut dirs_queue: VecDeque<PathBuf> = VecDeque::new();

    if src_dir.is_dir() {
        dirs_queue.push_back(src_dir.to_path_buf());
    }
    
    while dirs_queue.len() > 0 {
        let dir = dirs_queue.pop_front().unwrap();

        // Конструкция if let позволяет обрабатывать код толкьо если у нас результат Ok
        if let Ok(dir_content) = fs::read_dir(&dir){
            for entry in dir_content {
                // Проверяем, что это валидное значение
                let val = match entry {
                    Ok(val) => val,
                    Err(_) => continue
                };
    
                // Тогда его и обрабатываем
                let path = val.path();
                if path.is_dir() {
                    dirs_queue.push_back(path);
                } else {
                    paths.push(path);
                }
            }
        }else{
            println!("Failed to read dir: {:?}", &dir);
        }
    }
    return paths;
}


// one possible implementation of walking a directory only visiting files
/*fn visit_dirs(dir: &path::Path, cb: &dyn Fn(&std::path::PathBuf)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    }
    Ok(())
}*/
