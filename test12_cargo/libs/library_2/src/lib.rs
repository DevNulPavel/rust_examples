#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Таким образом мы указываем, что используем код из родительской области
        use super::*;

        assert_eq!(add_ten(20), 30);
        assert_eq!(add_ten(40), 50);
    }
}

pub fn add_ten(val: i32)-> i32{
    return val + 10;
}