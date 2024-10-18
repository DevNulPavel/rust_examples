#![warn(clippy::all)]

// Использование стороннего внешнего модуля
//extern mod dirs;

// Либо можно вот так (модифицируя путь)
#[path = "dirs.rs"]
mod dirs;


pub fn convert_to_spirv_shaders(input_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), String> {
    // Получаем путь к утилите конвертаци
    let cur_dir_path = std::env::current_dir().unwrap();
    let convert_util_app = if cfg!(target_os = "linux") {
        cur_dir_path.join("LinuxConverter").join("glslangValidator")
    } else {
        cur_dir_path.join("OSXConverter").join("glslangValidator")
    };
    drop(cur_dir_path);

    // Получаем список файлов
    let files = dirs::visit_dirs(input_path);

    let process_function = |file_path: &std::path::Path, 
                            input_path_local: &std::path::Path,
                            out_path_local: &std::path::Path,
                            convert_util_app_local: &std::path::Path| {
        // Получаем расширение
        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
        let file_extention = file_path.extension().unwrap().to_str().unwrap();
        if file_name.starts_with('.') || ((file_extention != "frag") && (file_extention != "vert")){
            return;
        }

        // Получаем папку
        let file_folder = file_path.parent().unwrap();

        // Получаем конечную папку
        let rel_path = file_folder.strip_prefix(&input_path_local).unwrap();
        let result_folder = out_path_local.join(rel_path);

        // И конечный путь
        let result_file_path = if file_extention == "vert" {
            let new_name = format!("{}_v", file_name);
            result_folder.join(file_name).with_file_name(new_name).with_extension("spv")
        } else if file_extention == "frag" {
            let new_name = format!("{}_p", file_name);
            result_folder.join(file_name).with_file_name(new_name).with_extension("spv")
        }else{
            return;
        };

        let out = std::process::Command::new(convert_util_app_local.to_str().unwrap())
            .arg("-V")
            .arg(file_path.to_str().unwrap())
            .arg("-o")
            .arg(result_file_path.to_str().unwrap())
            .output().unwrap();
        
        if out.status.code().unwrap() != 0 {
            let text = String::from_utf8(out.stdout).unwrap();
            println!("----- Shader compilation ERRROR! -----\n{}--------------------------------------\n", text);
        }
    };

    let num_threads = num_cpus::get();
    let mut threads_vec = Vec::with_capacity(num_threads);

    let (tx, rx) = crossbeam_channel::bounded(num_threads);
    for _ in 0..num_threads{
        let input_path_local = input_path.to_owned();
        let out_path_local = output_path.to_owned();
        let convert_util_app_local = convert_util_app.to_owned();

        let receiver = rx.clone();

        let thread_handle = std::thread::spawn(move || {
            let mut need_exit = false;
            while !need_exit {
                let received: Result<std::path::PathBuf, crossbeam_channel::RecvError> = receiver.recv();
                //println!("Message from thread: {}", i);
                match received {
                    Ok(file_path) => {
                        process_function(&file_path, &input_path_local, &out_path_local, &convert_util_app_local);
                    },
                    Err(_) => {
                        //println!("Need stop thread {}: {:?}", i, err); 
                        need_exit = true; 
                    },
                }
            }
        });
        threads_vec.push(thread_handle);
    }

    for file_path in files{
        tx.send(file_path).unwrap();
    }
    drop(tx);

    for joinable in threads_vec {
        joinable.join().unwrap();
    }

    Ok(())
}