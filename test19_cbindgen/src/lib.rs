#[no_mangle] 
pub extern fn function_1(param: i32) -> i32 {
    //println!("Value: {}", param);
    param
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
