mod macros;
mod numbers;
mod word;
mod float;

fn main() {
    numbers::parse_numbers();

    macros::test_macros();

    word::word();

    float::float();
}
