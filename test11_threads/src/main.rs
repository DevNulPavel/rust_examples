
mod park_thread_test;
mod threads_test;

fn main() {
    // threads_test::test_variables_move();
    // threads_test::test_channel_1();
    // threads_test::test_channel_2();

    park_thread_test::park_thread_test_1();
    park_thread_test::park_thread_test_2();
}