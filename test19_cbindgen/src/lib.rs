

/// cbindgen:field-names=[data, len]
/// cbindgen:derive-eq
#[repr(C)]
#[derive(Debug, Default)]
pub struct Buffer<T> {
    data_array: [T; 8],
    len: usize,
}

#[no_mangle] 
pub extern fn function_1(param: i32) -> i32 {
    println!("Value: {}", param);
    param
}

#[no_mangle] 
pub extern fn function_2(buffer: Buffer<i32>) -> i32 {
    println!("{:?}", buffer);
    let mut result = 0 as i32;
    for index in 0..buffer.len{
        if let Some(val) = buffer.data_array.get(index) {
            result += val;
        }else{
            break;
        }
    }
    result
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
