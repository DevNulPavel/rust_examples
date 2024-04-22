#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Таким образом мы указываем, что используем код из родительской области
        use super::*;

        assert_eq!(add_one(10), 11);
        assert_eq!(add_one(100), 101);
    }
}

pub fn add_one(val: i32)-> i32{
    return val + 1;
}
