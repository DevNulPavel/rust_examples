mod float;
mod macros_1;
mod macros_2;
mod numbers;
mod word;
mod by_ref;

fn main() {
    numbers::parse_numbers();

    macros_1::test_macros_1();

    macros_2::test_macros_2();

    word::word();

    float::float();

    by_ref::rest_ref();
}
