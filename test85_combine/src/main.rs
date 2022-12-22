mod macros_1;
mod macros_2;
mod numbers;
mod word;
mod float;

fn main() {
    numbers::parse_numbers();

    macros_1::test_macros();

    word::word();

    float::float();
}
