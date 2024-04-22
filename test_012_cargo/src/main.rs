extern crate library_1;
extern crate library_2;


fn main() {
    assert_eq!(library_1::add_one(5), 6);
    assert_eq!(library_2::add_ten(10), 20);
}
