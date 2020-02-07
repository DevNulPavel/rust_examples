extern crate lazy_static;
extern crate regex;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate num_cpus;

// Использование стороннего внешнего модуля
//extern mod dirs;

// Либо можно вот так (модифицируя путь)
// #[path = "dirs.rs"]
// mod dirs;
//mod crate::dirs; // Путь относительно корня
//mod super::dirs; // Путь относительно родителя

use crate::dirs; // Путь относительно корня
//use super::dirs; // Путь относительно родителя
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;


lazy_static!{
    static ref RE1: regex::Regex = regex::Regex::new(r"/\*.+\*/").unwrap();
    static ref RE2: regex::Regex = regex::Regex::new(r"/\*.+\*/\n").unwrap();
    static ref RE3: regex::Regex = regex::Regex::new(r"//.+\n").unwrap();
    static ref RE4: regex::Regex = regex::Regex::new(r"^\s*$").unwrap();
    static ref RE5: regex::Regex = regex::Regex::new(r"void main\s*\(void\)\s*\{[\s\S=]+\}").unwrap();
    static ref RE6: regex::Regex = regex::Regex::new(r"([a-zA-Z_]+)\[([0-9]+)\]").unwrap();
    static ref RE7: regex::Regex = regex::Regex::new(r"void main\s*\(void\)\s*\{").unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Таким образом описывается интерфейс конкретного объекта
pub trait ToString {
    // Описываем обязательные методы, кооторые надо реализовать
    fn to_string(&self) -> String;

    // Так же мы можем реализовать общий код для трейтов
    fn to_string_2(&self) -> String{
        self.to_string()
    }

    // Так же мы можем реализовать общий код для трейтов
    //fn to_string_3<T: ToString>(val: &T) -> String{
    //    return val.to_string();
    //}
}

// Таким вот образом можно автоматически реализовать методы для клонирования
// https://rurust.github.io/rust-by-example-ru/trait/derive.html
#[derive(Clone, Default, Debug)]
struct ShaderProcessResult{
    set_index: i32, 
    push_constants_offset: i32,
    result_varying_locations: HashMap<String, i32>
}

// Реализуем перевод в строку для ShaderProcessResult
/*impl ToString for ShaderProcessResult{
    fn to_string(&self) -> String{
        return format!("{}", self);
    }
    // fn to_string_2(&self) -> String{
    //     return format!("{}", self);
    // }
}*/

// Либо сразу реализуем для всех типов T, поддерживающих std::fmt::Display, перевод в строку
// Поэтому даже такой код будет валиден: let s = 3.to_string();
impl<T> ToString for T
    where T: Display 
{
    fn to_string(&self) -> String{
        return format!("{}", self);
    }
}

// Так мы реализуем стандартный интерфейс форматтера
impl Display for ShaderProcessResult{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut varyings_text = String::new();
        if !self.result_varying_locations.is_empty() {
            varyings_text.push_str("{");
            for (key, index) in self.result_varying_locations.iter() {
                let text = format!("{}: {}, ", key, index);
                varyings_text.push_str(text.as_str());
            }
            varyings_text.pop();
            varyings_text.pop();
            varyings_text.push_str("}");
        }else{
            varyings_text.push_str("{}");
        }
        return write!(f, "Index: {}, push_offset: {}, varyings: {}", self.set_index, self.push_constants_offset, varyings_text);
    }    
}

// Реализуем возможность работать с содержимым объекта без разыменования его
impl Deref for ShaderProcessResult{
    type Target = HashMap<String, i32>; // Указываем тип

    /*fn deref(&self) -> &std::collections::HashMap<String, i32> {
        return &self.result_varying_locations;
    }*/
    fn deref(&self) -> &Self::Target {
        &self.result_varying_locations
    }
}

impl ShaderProcessResult{
    fn get_set_index(&self) -> i32{
        self.set_index
    }
    fn get_push_constants_offset(&self) -> i32{
        self.push_constants_offset
    }
    fn get_result_varying_locations(&self) -> &HashMap<String, i32>{
        //return &self.result_varying_locations;
        &self // Пользуемся trait Deref выше, чтобы работать с содержимым как есть
    }
}

// Указываем, что шаблон должен реализовывать trait
fn print_shader_process_result_1<T: ToString + Display>(val: &T){
    println!("{}", val.to_string());
}

// dyn - нужен для того, чтобы можно было прокидывать любой объект, реализующий trait
/*fn print_shader_process_result_2(val: &dyn ToString){
    println!("{}", val.to_string());
}*/

// Указываем, что шаблон должен реализовывать trait, но можно указывать альтернативным способом
/*fn print_shader_process_result_3<T>(val: &T) where T: ToString + std::fmt::Display {
    println!("{}", val.to_string());
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn convert_type_to_size(type_name: &str)->i32{
    match type_name {
        "bool" => 1,
        "int" | "float" => 4,
        "vec2" => 4*2,
        "vec3" => 4*3,
        "vec4" => 4*4,
        "mat2" => 4*2*2,
        "mat3" => 4*3*3,
        "mat4" => 4*4*4,
        _ => 0,
    }
}

fn convert_type_to_sort_priority(type_name: &str)->i32{
    match type_name {
        "bool" => 1,
        "int" => 2,
        "float" => 3,
        "vec2" => 4,
        "vec3" => 5,
        "vec4" => 6,
        "mat2" => 7,
        "mat3" => 8,
        "mat4" => 9,
        _ => 0,
    }
}

fn analyze_uniforms_size(file_path: &std::path::Path) -> i32 {
    use std::io::Read;

    // TODO: в функцию
    let mut file = std::fs::File::open(file_path).unwrap();
    let mut input_file_text = String::new();
    file.read_to_string(&mut input_file_text).unwrap();
    drop(file);
    
    // Табы заменяем на пробелы
    input_file_text = input_file_text.replace("\t", "    ");

    // Удаляем комментарии
    input_file_text = String::from(RE1.replace_all(input_file_text.as_str(), ""));
    input_file_text = String::from(RE2.replace_all(input_file_text.as_str(), "\n"));
    input_file_text = String::from(RE3.replace_all(input_file_text.as_str(), "\n"));

    // Удаляем странные дефайны
    input_file_text = input_file_text.replace("FLOAT", "float");

    // Удаляем пустые строки (flags=re.MULTILINE)
    input_file_text = String::from(RE4.replace_all(input_file_text.as_str(), ""));

    input_file_text = input_file_text.replace("\n", " ");
    let raw_words: Vec<&str> = input_file_text.split(' ').collect();
    let words: Vec<&str> = raw_words.into_iter().filter(|val| !val.is_empty() ).collect();

    let mut main_func_text = String::from(RE5.captures(input_file_text.as_str()).unwrap().get(0).unwrap().as_str());

    // Remove precision words
    let precision_words: [&str; 7] = ["lowp", "mediump", "highp", "PRECISION_LOW", "PRECISION_MEDIUM", "PRECISION_HIGH", "PLATFORM_PRECISION"];
    for word in precision_words.iter(){
        main_func_text = main_func_text.replace(word, "PRECISION");
    }
    main_func_text = main_func_text.replace("PRECISION ", "");

    //println!("{:?}\n", main_func_text);

    let mut uniforms_size: i32 = 0;
    let mut uniforms_vec : Vec<String> = Vec::new();
    let mut uniforms_map = std::collections::HashMap::new();

    let mut i: usize = 0;
    while i < words.len() {
        if words[i] != "uniform"{
            i += 1;
            continue;
        }

        // Если после uniform идет описание точности - пропускаем его
        i += 1;
        if precision_words.contains(&words[i]) {
            i += 1;
        }
        
        // Если после uniform идет sampler2D - не обрабоатываем
        if words[i] == "sampler2D"{
            i += 1;
            continue;
        }

        // Тип аттрибута
        let uniform_type = String::from(words[i]);

        // Получаем имя аттрибута
        i += 1;
        let test_uniform_name = words[i].replace(";", "");
        
        if !uniforms_vec.contains(&test_uniform_name) && main_func_text.contains(test_uniform_name.as_str()){
            uniforms_size += convert_type_to_size(uniform_type.as_str());
            uniforms_vec.push(test_uniform_name.clone());
            uniforms_map.insert(test_uniform_name, uniform_type);
        }
    }

    uniforms_size
}

fn read_file(input_path: &std::path::Path)-> Result<String, String>{
    use std::io::Read;
    use std::fs::File;
    
    // Читаем файл                     
    let mut file = File::open(input_path)
        .or_else(|err|->Result<File, String>{
            Err(format!("Failed to read file: {:?}, err: {:?}", input_path, err))
        })?;

    let mut input_file_text = String::new();
    if let Err(err) = file.read_to_string(&mut input_file_text){
        return Err(format!("Failed to read content of file: {:?}, err: {:?}", input_path, err));
    }
    drop(file);

    Ok(input_file_text)
}

fn prepare_text<'a>(input_path: &std::path::Path, 
                    input_file_text: &'a mut String, 
                    precision_words: [&str; 7])-> Result<(&'a String, String, Vec<&'a str>), String>{
    // Табы заменяем на пробелы
    input_file_text.clone_from(&input_file_text.replace("\t", "    "));

    // Удаляем комментарии
    input_file_text.clone_from(&String::from(RE1.replace_all(input_file_text.as_str(), "")));
    input_file_text.clone_from(&String::from(RE2.replace_all(input_file_text.as_str(), "\n")));
    input_file_text.clone_from(&String::from(RE3.replace_all(input_file_text.as_str(), "\n")));

    // Удаляем странные дефайны
    input_file_text.clone_from(&input_file_text.replace("FLOAT", "float"));

    let mut main_func_text = String::from(match RE5.captures(input_file_text.as_str()) {
        Some(text)=> match text.get(0) {
            Some(main_func_text)=> {
                main_func_text.as_str()
            },
            None=> {
                return Err(format!("Failed to parse main function at file: {:?}", input_path));    
            }
        },
        None=>{
            return Err(format!("Failed to parse main function at file: {:?}", input_path));
        }
    });

    input_file_text.clone_from(&input_file_text.replace("\n", " "));
    let raw_words: Vec<&str> = input_file_text.split(' ').collect();
    let words: Vec<&'a str> = raw_words.into_iter()
        .filter(|val| {
            !val.is_empty()
        })
        .collect();

    // Remove precision words
    for word_s in precision_words.iter() {
        let word = word_s as &str;
        main_func_text = main_func_text.replace(word, "PRECISION");
    }
    main_func_text = main_func_text.replace("PRECISION ", "");

    Ok((input_file_text, main_func_text, words))
}

fn process_shader_file( is_vertex_shader: bool, 
                        input_path: &std::path::Path,
                        out_path: &std::path::Path, 
                        src_set_index: i32,
                        input_varying_locations: &std::collections::HashMap<String, i32>, 
                        src_push_constants_offset: i32, 
                        use_push_constants: bool) -> Result<ShaderProcessResult, String> {
    use std::fs::File;                            
    use std::io::Read;
    use std::io::Write;
    use std::collections::HashMap;

    // Читаем файл
    let mut input_file_text = read_file(input_path)?;

    let precision_words = ["lowp", "mediump", "highp", "PRECISION_LOW", "PRECISION_MEDIUM", "PRECISION_HIGH", "PLATFORM_PRECISION"];
    let (input_file_text, mut main_func_text, words)  = prepare_text(input_path, &mut input_file_text, precision_words)?;

    // Ignore defines
    let ignore_defines_list: [&str; 3] = ["float", "FLOAT", "VEC2"];

    // Lists
    let mut defines_names_list: Vec<String> = vec![];
    let mut defines_map: HashMap<String, String> = HashMap::new();
    let mut attributes_names_list: Vec<String> = vec![];
    let mut attributes_map = HashMap::new();
    let mut uniforms_names_list: Vec<String> = vec![];
    let mut uniforms_map: HashMap<String, HashMap<&str, String>> = HashMap::new();
    let mut varyings_names_list: Vec<String> = vec![];
    let mut varyings_map: HashMap<String, String> = HashMap::new();
    let mut samplers_names_list: Vec<String> = vec![];

    let mut result_varying_locations = HashMap::new();

    // Обходим слова и ищем аттрибуты
    let mut attribute_index = 0 as i32;
    let mut varying_index = 0 as i32;
    let mut i = 0 as usize;
    while i < words.len(){
        if words[i] == "#define"{
            // Получаем тип аттрибута
            i += 1;
            let define_name = String::from(words[i]);

            if main_func_text.contains(define_name.as_str()) && !ignore_defines_list.contains(&define_name.as_str()){
                i += 1;
                let define_value = String::from(words[i]);
                // Добавляем аттрибут к тексту нового шейдера
                if !defines_names_list.contains(&define_name) {
                    let new_shader_define_name = format!("#define {} {}\n", define_name, define_value);
                    defines_map.insert(define_name.clone(), new_shader_define_name); // TODO: ???
                    defines_names_list.push(define_name);
                }
            }
        }

        if words[i] == "attribute" {
            // Если после attribute идет описание точности - пропускаем его
            i += 1;
            if precision_words.contains(&words[i]){
                i += 1;
            }

            // Получаем тип аттрибута
            let attribute_type = &words[i];

            // Получаем имя аттрибута
            i += 1;
            let attribute_name = words[i].replace(";", "");

            // Добавляем аттрибут к тексту нового шейдера
            if !attributes_names_list.contains(&attribute_name){
                let new_shader_variable_name = format!("layout(location = {}) in {} {};\n", attribute_index, attribute_type, attribute_name);
                attributes_map.insert(attribute_name.clone(), new_shader_variable_name); // TODO: ???
                attributes_names_list.push(attribute_name);
                attribute_index += 1;
            }
        }

        if words[i] == "uniform" {
            // Если после uniform идет описание точности - пропускаем его
            i += 1;
            if precision_words.contains(&words[i]){
                i += 1;
            }

            // Если после uniform идет sampler2D - не обрабоатываем
            if words[i] != "sampler2D"{
                // Получаем тип униформа
                let uniform_type = words[i];
                
                // Получаем имя униформа
                i += 1;
                let uniform_name = words[i].replace(";", "");

                // Добавляем юниформ к тексту нового шейдера
                if !uniforms_names_list.contains(&uniform_name){
                    if main_func_text.contains(&uniform_name){
                        let mut uniform_info = std::collections::HashMap::new();
                        uniform_info.insert("type", String::from(uniform_type));
                        uniform_info.insert("name", String::from(&uniform_name));
                        uniforms_map.insert(String::from(&uniform_name), uniform_info);
                        uniforms_names_list.push(uniform_name);
                    }else{
                        println!("Unused uniform variable {:?} in shader {:?}", uniform_name, input_path);
                    }
                }
            }else{
                // Получаем имя семплера
                i += 1;
                let sampler_name = words[i].replace(";", "");

                // Добавляем аттрибут к тексту нового шейдера
                if !samplers_names_list.contains(&sampler_name){
                    samplers_names_list.push(sampler_name);
                }
            }
        }

        if words[i] == "varying" {
            // Если после varying идет описание точности - пропускаем его
            i += 1;
            if precision_words.contains(&words[i]){
                i += 1;
            }

            // Получаем тип
            let varying_type = &words[i];

            // Получаем имя
            i += 1;
            let varying_name = words[i].replace(";", "");

            // Может быть у нас массив varying переменных
            if varying_name.contains("["){
                let varying_matches = RE6.captures(varying_name.as_str());
                match varying_matches{
                    Some(found) => {
                        let name = found.get(1).unwrap().as_str();
                        let count: i32 = found.get(2).unwrap().as_str().parse().unwrap();
                        //println!("{:?} {:?}", name, count);

                        if main_func_text.contains(name){
                            for index in 0..count{
                                let varying_count_name = format!("{}_{}", name, index);

                                let mut new_shader_var_name = String::new();
                                if !varyings_names_list.contains(&varying_count_name){
                                    if is_vertex_shader {
                                        result_varying_locations.insert(varying_count_name.clone(), varying_index as i32);
                                        new_shader_var_name = format!("layout(location = {}) out {} {};\n", varying_index, varying_type, varying_count_name);
                                    }else{
                                        if input_varying_locations.contains_key(&varying_count_name){
                                            let input_index = input_varying_locations.get(&varying_count_name).unwrap();
                                            new_shader_var_name = format!("layout(location = {}) in {} {};\n", input_index, varying_type, varying_count_name);
                                        }else{
                                            println!("Is not compatible varyings for {:?} with vertex shader", input_path);   
                                        }
                                    }

                                    varyings_map.insert(varying_count_name.clone(), new_shader_var_name);
                                    varyings_names_list.push(varying_count_name.clone());
                                    varying_index += 1;

                                    // TODO: ????
                                    // Замена в тексте нашей переменной
                                    let rex_exp_format = format!(r"{}\[[ ]*{}[ ]*\]", name, index);
                                    let reg_exp: regex::Regex = regex::Regex::new(&rex_exp_format).unwrap();
                                    let count_var_str = varying_count_name.as_str();
                                    main_func_text = String::from(reg_exp.replace_all(&main_func_text, count_var_str));
                                }
                            }
                        }else{
                            // TODO: ???
                        }
                    },
                    None => {
                    },
                }
            }else{
                // Добавляем аттрибут к тексту нового шейдера
                if !varyings_names_list.contains(&varying_name) && main_func_text.contains(&varying_name){
                    let mut new_shader_var_name = String::new();
                    if is_vertex_shader {
                        result_varying_locations.insert(varying_name.clone(), varying_index as i32);
                        new_shader_var_name = format!("layout(location = {}) out {} {};\n", varying_index, varying_type, varying_name);
                    }else{
                        if input_varying_locations.contains_key(&varying_name){
                            let input_index = input_varying_locations.get(&varying_name).unwrap();
                            new_shader_var_name = format!("layout(location = {}) in {} {};\n", input_index, varying_type, varying_name);
                        }else{
                            println!("Is not compatible varyings for {:?} with vertex shader", input_path);   
                        }
                    }

                    varyings_map.insert(varying_name.clone(), new_shader_var_name);
                    varyings_names_list.push(varying_name);
                    varying_index += 1;
                }
            }
        }

        i += 1;
    }

    //println!("{:?} {:?}", varyings_names_list, varyings_map);
    //println!("{:?}", samplers_names_list);

    let mut set_index = src_set_index;
    let mut push_constants_offset = src_push_constants_offset;

    let mut result_shader_text = String::new();
    result_shader_text.push_str("#version 450\n\n");
    result_shader_text.push_str("#extension GL_ARB_separate_shader_objects : enable\n\n");

    if is_vertex_shader{
        result_shader_text.push_str("// Vertex shader\n\n");

        // Defines
        if defines_map.len() > 0{
            result_shader_text.push_str("// Defines\n");
            for (_, val) in defines_map.iter(){
                result_shader_text.push_str(val);
            }
            result_shader_text.push_str("\n");
        }
        
        // Attributes
        if attributes_map.len() > 0{
            result_shader_text.push_str("// Input\n");
            for (_, val) in attributes_map.iter(){
                result_shader_text.push_str(val);
            }
            result_shader_text.push_str("\n");
        }

        // Uniforms
        if use_push_constants{
            if uniforms_names_list.len() > 0{
                // Сортируем по убыванию размера
                uniforms_names_list.sort_by(|a, b| {
                    let prior1 = convert_type_to_sort_priority(uniforms_map.get(a).unwrap().get("type").unwrap());
                    let prior2 = convert_type_to_sort_priority(uniforms_map.get(b).unwrap().get("type").unwrap());
                    if prior1 < prior2{
                        return std::cmp::Ordering::Greater;
                    }else if prior1 > prior2{
                        return std::cmp::Ordering::Less;
                    }
                    return std::cmp::Ordering::Equal;
                });

                let mut push_constants_text = String::new();

                for uniform_name in &uniforms_names_list{
                    let uniform_dict = uniforms_map.get(uniform_name).unwrap();
                    let un_type = uniform_dict.get("type").unwrap();
                    let un_name = uniform_name;
                    let new_shader_var_name = format!("    layout(offset = {}) {} {};\n", push_constants_offset, un_type, un_name);
                    push_constants_text.push_str(&new_shader_var_name);
                    push_constants_offset += convert_type_to_size(un_type);
                }

                if push_constants_text.len() > 0{
                    result_shader_text.push_str("// Push constants\n");
                    result_shader_text.push_str("layout(push_constant) uniform PushConstants {\n");
                    result_shader_text.push_str(&push_constants_text);
                    result_shader_text.push_str("} uni;\n\n");
                }

                //println!("{:?}", result_shader_text);
            }
        }else{
            // Сортируем по убыванию размера
            uniforms_names_list.sort_by(|a, b| {
                let prior1 = convert_type_to_sort_priority(uniforms_map.get(a).unwrap().get("type").unwrap());
                let prior2 = convert_type_to_sort_priority(uniforms_map.get(b).unwrap().get("type").unwrap());
                if prior1 < prior2{
                    return std::cmp::Ordering::Greater;
                }else if prior1 > prior2{
                    return std::cmp::Ordering::Less;
                }
                return std::cmp::Ordering::Equal;
            });

            let mut uniforms_text = String::new();

            for uniform_name in &uniforms_names_list{
                let uniform_dict = uniforms_map.get(uniform_name).unwrap();
                let un_type = uniform_dict.get("type").unwrap();
                let un_name = uniform_name;
                let new_shader_var_name = format!("    {} {};\n" , un_type, un_name);
                uniforms_text.push_str(&new_shader_var_name);
            }

            if uniforms_text.len() > 0{
                result_shader_text.push_str("// Uniform buffer\n");
                result_shader_text.push_str("layout(set = 0, binding = 0) uniform UniformBufferObject {\n");
                result_shader_text.push_str(&uniforms_text);
                result_shader_text.push_str("} uni;\n\n");
            }

            //println!("{:?}", result_shader_text);
        }

        // Varyings
        if varyings_map.len() > 0{
            result_shader_text.push_str("// Varying variables\n");
            for (_, val) in varyings_map.iter(){
                result_shader_text.push_str(val);
            }
            result_shader_text.push_str("\n");
        }

        // Выходные переменные
        result_shader_text.push_str("// Vertex output\n");
        result_shader_text.push_str("out gl_PerVertex {\n");
        result_shader_text.push_str("    vec4 gl_Position;\n");
        result_shader_text.push_str("};\n");
    }else{
        result_shader_text.push_str("// Fragment shader\n\n");

        // Defines
        if defines_map.len() > 0{
            result_shader_text.push_str("// Defines\n");
            for (_, val) in defines_map.iter(){
                result_shader_text.push_str(val);
            }
            result_shader_text.push_str("\n");
        }
        
        // Varying
        if varyings_map.len() > 0{
            result_shader_text.push_str("// Varying variables\n");
            for (_, val) in varyings_map.iter(){
                result_shader_text.push_str(val);
            }
            result_shader_text.push_str("\n");
        }

        // Uniforms
        if use_push_constants{
            if uniforms_names_list.len() > 0{
                // Сортируем по убыванию размера
                uniforms_names_list.sort_by(|a, b| {
                    let prior1 = convert_type_to_sort_priority(uniforms_map.get(a).unwrap().get("type").unwrap());
                    let prior2 = convert_type_to_sort_priority(uniforms_map.get(b).unwrap().get("type").unwrap());
                    if prior1 < prior2{
                        return std::cmp::Ordering::Greater;
                    }else if prior1 > prior2{
                        return std::cmp::Ordering::Less;
                    }
                    return std::cmp::Ordering::Equal;
                });

                let mut push_constants_text = String::new();

                for uniform_name in &uniforms_names_list{
                    let uniform_dict = uniforms_map.get(uniform_name).unwrap();
                    let un_type = uniform_dict.get("type").unwrap();
                    let un_name = uniform_name;
                    let new_shader_var_name = format!("    layout(offset = {}) {} {};\n", push_constants_offset, un_type, un_name);
                    push_constants_text.push_str(&new_shader_var_name);
                    push_constants_offset += convert_type_to_size(un_type);
                }

                if push_constants_text.len() > 0{
                    result_shader_text.push_str("// Push constants\n");
                    result_shader_text.push_str("layout(push_constant) uniform PushConstants {\n");
                    result_shader_text.push_str(&push_constants_text);
                    result_shader_text.push_str("} uni;\n\n");
                }

                //println!("{:?}", result_shader_text);
            }
        }else{
            // Сортируем по убыванию размера
            uniforms_names_list.sort_by(|a, b| {
                let prior1 = convert_type_to_sort_priority(uniforms_map.get(a).unwrap().get("type").unwrap());
                let prior2 = convert_type_to_sort_priority(uniforms_map.get(b).unwrap().get("type").unwrap());
                if prior1 < prior2{
                    return std::cmp::Ordering::Greater;
                }else if prior1 > prior2{
                    return std::cmp::Ordering::Less;
                }
                return std::cmp::Ordering::Equal;
            });

            let mut uniforms_text = String::new();

            for uniform_name in &uniforms_names_list{
                let uniform_dict = uniforms_map.get(uniform_name).unwrap();
                let un_type = uniform_dict.get("type").unwrap();
                let un_name = uniform_name;
                let new_shader_var_name = format!("    {} {};\n" , un_type, un_name);
                uniforms_text.push_str(&new_shader_var_name);
            }

            if uniforms_text.len() > 0{
                result_shader_text.push_str("// Uniform buffer\n");
                result_shader_text.push_str("layout(set = 0, binding = 0) uniform UniformBufferObject {\n");
                result_shader_text.push_str(&uniforms_text);
                result_shader_text.push_str("} uni;\n\n");

                set_index += 1;
            }

            //println!("{:?}", result_shader_text);
        }

        // Samplers
        if samplers_names_list.len() > 0{
            result_shader_text.push_str( "// Samplers\n");
            for sampler_name in samplers_names_list.iter(){
                let sampler_text = format!("layout(set = {}, binding = 0) uniform sampler2D {};\n", set_index, sampler_name);
                result_shader_text.push_str(&sampler_text);
                set_index += 1;
            }
            result_shader_text.push_str("\n");
        }

        // Выходные переменные
        result_shader_text.push_str("// Fragment output\n");
        result_shader_text.push_str("layout(location = 0) out vec4 outputFragColor;\n");
    }

    let found = RE7.captures(&main_func_text).unwrap();
    let main_func_decl = String::from(&found[0]);

    // Function declaration replace
    main_func_text = main_func_text.replace(&main_func_decl, "void main(void) {");
    
    // Replace uniforms on push constants    
    for uniform_name in &uniforms_names_list{
        // [\+\-\ * \ /(<>=]({})[\+\-\ * \ /, ;.\[)<>=]
        let rex_exp_format = format!(r"[\+\-\*/<>=\s(]({})[\+\-\*/<>=;,.\s)]", uniform_name); // TODO: ???
        let reg_exp: regex::Regex = regex::Regex::new(&rex_exp_format).unwrap();
        let new_val = format!("uni.{}", uniform_name);

        loop {
            let matches = reg_exp.captures(&main_func_text);
            match matches{
                Some(found) => {
                    let mut buffer = String::new();
                    let gr = found.get(1).unwrap();
                    let start = gr.start();
                    let stop = gr.end();
                    buffer.push_str(&main_func_text[..start]);
                    buffer.push_str(&new_val);
                    buffer.push_str(&main_func_text[stop..]);
                    main_func_text = buffer;
                },
                None => {
                    break;
                },
            }
        }
    }

    //println!("{:?}", main_func_text);

    // Fragment out variable
    if !is_vertex_shader {
        main_func_text = main_func_text.replace("texture2D", "texture");
        main_func_text = main_func_text.replace("gl_FragColor", "outputFragColor");
    }

    result_shader_text.push_str("\n// Main function\n");
    result_shader_text.push_str(&main_func_text);

    // Выравнивание
    if push_constants_offset % 16 != 0{
        push_constants_offset += 16;
        push_constants_offset -= push_constants_offset % 16;
    }

    // Сохраняем
    let mut res_file = std::fs::File::create(out_path).unwrap();
    let bytes = result_shader_text.as_bytes();
    res_file.write_all(&bytes).unwrap();
    drop(res_file);

    return Ok(ShaderProcessResult {
        set_index, 
        push_constants_offset, 
        result_varying_locations
    });
}

fn pre_process_shader_file(file_path:   &std::path::Path, 
                           input_path:  &std::path::Path, 
                           out_path:    &std::path::Path) -> Result<(), String> 
{
    use std::path::Path;
    use std::path::PathBuf;

    // Конвертируем в имя
    let file_name = match file_path.file_name() {
        Some(file_name)=> match file_name.to_str() {
            Some(file_name_str)=> file_name_str,
            None => return Err(format!("Invalid file name convertation to str: {:?}", file_path)),    
        },
        None => return Err(format!("Invalid file path: {:?}", file_path)),
    };

    // Конвертируем в расширение
    let file_extention = match file_path.extension() {
        Some(file_extention)=> match file_extention.to_str() {
            Some(file_extention_str)=> file_extention_str,
            None => return Err(format!("Invalid file extention convertation to str: {:?}", file_path)),
        },
        None => return Err(format!("Invalid file extention: {:?}", file_path)),
    };

    // Отбрасываем неподходящие файлы
    if file_name.starts_with(".") || file_extention != "psh"{
        return Ok(());
    }

    // Получаем папку файлика
    let file_folder = match file_path.parent(){
        Some(file_folder) => file_folder,
        None => return Err(format!("Missing parent at filepath: {:?}", file_path)),
    };

    // Получаем конечную папку
    let path_buf = PathBuf::from(file_folder);
    let rel_path = match path_buf.strip_prefix(&input_path){
        Ok(rel_path) => rel_path,
        Err(err)=> return Err(format!("Prefix strip failed: {:?}, error: {:?}", file_path, err)),
    };
    let result_folder = out_path.join(rel_path);

    // Получаем пути для начального и конечного файлов
    let source_fragment_file_path = Path::new(&file_path);
    let source_vertex_file_path = source_fragment_file_path.with_extension("vsh");
    let result_fragment_file_path = result_folder.join(file_name).with_extension("frag");
    let result_vertex_file_path = result_fragment_file_path.with_extension("vert");

    // Проверка наличия всех нужных файлов
    if !source_fragment_file_path.exists() || !source_vertex_file_path.exists(){
        let text = format!("Missing shaders {:?} + {:?}", source_fragment_file_path, source_vertex_file_path);
        return Err(text);
    }

    let vertex_uniforms_size = analyze_uniforms_size(&source_vertex_file_path);
    let fragment_uniforms_size = analyze_uniforms_size(&source_fragment_file_path);
    //println!("{:?} {:?}\n", fragment_uniforms_size, fragment_uniforms);

    let total_uniforms_size = vertex_uniforms_size + fragment_uniforms_size;

    //let mut full_uniforms_dict = std::collections::HashMap::new();

    let mut use_push_constants = false;
    if total_uniforms_size <= 128{
        use_push_constants = true;
    }else{
        //full_uniforms_dict.extend(vertex_uniforms);
        //full_uniforms_dict.extend(fragment_uniforms);
    }

    // Обработка вершинного шейдера
    let vertex_result = process_shader_file(
        true, 
        &source_vertex_file_path, 
        &result_vertex_file_path, 
        0,
        &std::collections::HashMap::new(), 
        0, 
        use_push_constants);

    // Обработка результата
    let vertex_info = match vertex_result{
        Ok(vertex_info) => vertex_info,
        Err(err)=> return Err(format!("Failed process file: {:?}, error: {:?}", source_vertex_file_path, err)),
    };

    print_shader_process_result_1(&vertex_info);

    // Обработка фрагментного шейдера
    let fragment_result = process_shader_file(
        false, 
        &source_fragment_file_path, 
        &result_fragment_file_path,
        vertex_info.get_set_index(),
        vertex_info.get_result_varying_locations(), 
        vertex_info.get_push_constants_offset(), 
        use_push_constants);

    // Обработка результата
    match fragment_result{
        Ok(_) => {},
        Err(err)=> return Err(format!("Failed process file: {:?}, error: {:?}", source_fragment_file_path, err)),
    };
        
    return Ok(());
}

pub fn pre_process_shader_folder(input_path: &std::path::Path, out_path: &std::path::Path) -> Result<(), String>{
    use std::path::PathBuf;
    use std::thread;
    use std::thread::JoinHandle;
    use std::sync::{Arc, Mutex};
    use crossbeam_channel::bounded;
    use crossbeam_channel::{Sender, Receiver};
    //use std::sync::mpsc::channel;
    //use std::thread::Thread;

    let paths: Vec<PathBuf> = dirs::visit_dirs(input_path);

    // Список потоков
    let mut threads_vec: Vec<JoinHandle<()>> = Vec::new();

    // Получаем количество ядер + создаем канал
    let num_threads = num_cpus::get();
    let (tx, rx): (Sender<PathBuf>, Receiver<PathBuf>) = bounded(num_threads);

    let base_lock = Arc::new(Mutex::new(0));
    
    for _ in 0..num_threads{
        // Создаем копии путей для прокидывания в поток
        let input_path_local: std::path::PathBuf = input_path.to_path_buf();
        let out_path_local: std::path::PathBuf = out_path.to_path_buf();

        // Копия канала получателя
        let receiver = rx.clone();

        // Блокировка
        let thread_lock = base_lock.clone();

        // Создаем поток
        let thread_handle = thread::spawn(move || {
            // Получаем из канала значения, пока у Result не будет Err
            while let Ok(path_val) = receiver.recv() {
                // Конвертируем файлик
                let res = pre_process_shader_file(&path_val, &input_path_local, &out_path_local);

                // Обрабатываем только наличие ошибки
                if let Err(err) = res {
                    let _guard = thread_lock.lock().unwrap(); // Блокировка одновременного доступа к выводу
                    println!("{}", err);
                }
            }
        });
        threads_vec.push(thread_handle);
    }

    // Передаем файлики через канал
    for file_path in paths{
        let send_res = tx.send(file_path);
        if let Err(err) = send_res {
            let _guard = base_lock.lock().unwrap();
            println!("Channel send err: {}", err)
        }        
    }

    // Уничтожаем канал, чтобы уничтожились потоки тоже
    drop(tx);

    // Цепляемся к потокам, чтобы точно завершились
    for joinable in threads_vec {
        let join_res = joinable.join();
        if let Err(err) = join_res {
            let _guard = base_lock.lock().unwrap();
            println!("Channel send err: {:?}", err);
        }
    }

    // Блокировка больше не нужна
    drop(base_lock);

    return Ok(());
}