use std::io;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;

fn test_file_read(){
    let open_file_res: std::result::Result<std::fs::File, std::io::Error> = File::open("hello.txt");

    let mut f = match open_file_res {
        Ok(file) => { 
            file 
        },
        Err(ref error) if (error.kind() == ErrorKind::NotFound) => {
            let file_create_res = File::create("hello.txt");
            match file_create_res {
                Ok(fc) => { 
                    fc
                },
                Err(e) => {
                    panic!("Tried to create file but there was a problem: {:?}", e)
                }
            }
        },
        Err(error) => {
            panic!("There was a problem opening the file: {:?}", error);
        },
    };
    println!("{:#?}", &f);
    let mut file_content = String::new();
    let read_result = f.read_to_string(&mut file_content);
    
    let bytes = match read_result{
        Ok(count) => {
            println!("File content: {}, {} bytes", &file_content, count);
            count
        },
        Err(_) => {
            println!("File read failed");
            0
        }
    };    
}

fn short_error_check() -> Result<String, io::Error> {
    // Сокращенный вариант записи c ? в конце, который автоматически возвращает ошибку из функции
    let mut f2 = File::open("hello.txt")?;
    let mut s2 = String::new();
    f2.read_to_string(&mut s2)?;
    
    return Ok(s2); // Возвращаем результат Result<String, io::Error>(s2, None)
}

fn main() {
    test_file_read();
}