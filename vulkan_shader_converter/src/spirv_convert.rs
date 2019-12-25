// Использование стороннего внешнего модуля
//extern mod dirs;

// Либо можно вот так (модифицируя путь)
#[path = "dirs.rs"]
mod dirs;


pub fn convert_to_spirv_shaders(input_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), String> {
    let cur_dir_path = std::env::current_dir().unwrap();
    let mut convert_util_app = std::path::PathBuf::from(&cur_dir_path);
    if cfg!(target_os = "linux") {
        convert_util_app = convert_util_app.join("LinuxConverter").join("glslangValidator");
    } else {
        convert_util_app = convert_util_app.join("OSXConverter").join("glslangValidator");
    };

    let files = dirs::visit_dirs(input_path);

    let process_function = |file_path: std::path::PathBuf, 
                            input_path_local: &std::path::Path,
                            out_path_local: &std::path::Path,
                            convert_util_app_local: &std::path::Path| {
                                
        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
        let file_extention = file_path.extension().unwrap().to_str().unwrap();
        if file_name.starts_with(".") || ((file_extention != "frag") && (file_extention != "vert")){
            return;
        }

        let file_folder = file_path.parent().unwrap();

        let path_buf = std::path::PathBuf::from(file_folder);
        let rel_path = path_buf.strip_prefix(&input_path_local).unwrap();
        let result_folder = out_path_local.join(rel_path);

        let mut result_file_path = result_folder.join(file_name);
        if file_extention == "vert" {
            let new_name = format!("{}_v", file_name);
            result_file_path = result_file_path.with_file_name(new_name).with_extension("spv");
        } else if file_extention == "frag" {
            let new_name = format!("{}_p", file_name);
            result_file_path = result_file_path.with_file_name(new_name).with_extension("spv");
        }

        let out = std::process::Command::new(convert_util_app_local.to_str().unwrap()).
            arg("-V").
            arg(file_path.to_str().unwrap()).
            arg("-o").
            arg(result_file_path.to_str().unwrap()).
            output().unwrap();
        if out.status.code().unwrap() != 0{
            let text = String::from_utf8(out.stdout).unwrap();
            println!("----- Shader compilation ERRROR! -----\n{}--------------------------------------\n", text);
        }
    };

    let mut threads_vec = Vec::new();

    let num_threads = num_cpus::get();
    let (tx, rx) = crossbeam_channel::bounded(num_threads);
    for _ in 0..num_threads{
        //let input_path_local = std::path::Path::new(input_path);
        let input_path_local = input_path.to_owned();
        let out_path_local = output_path.to_owned();
        let convert_util_app_local = convert_util_app.to_owned();
        let receiver = rx.clone();
        let thread_handle = std::thread::spawn(move || {
            let mut need_exit = false;
            while need_exit == false {
                let received: Result<std::path::PathBuf, crossbeam_channel::RecvError> = receiver.recv();
                //println!("Message from thread: {}", i);
                match received {
                    Ok(file_path) => {
                        process_function(file_path, &input_path_local, &out_path_local, &convert_util_app_local);
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