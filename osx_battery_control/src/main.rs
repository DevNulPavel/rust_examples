// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
mod bindings;

fn main(){
    unsafe{
        // bindings::smc_init();
        // bindings::smc_close();

        // use libc::c_char;
        // use std::ffi::CString;
        // use std::slice::i;

        let mut conn: IOKit_sys::io_connect_t = IOKit_sys::io_connect_t::default();

        let res1 = bindings::SMCOpen(&mut conn);
        println!("Res1: {}", res1);

        println!("Conn: {}", conn);

        let key = bindings::UInt32Char_t{
            // data: [0_i8; 5_usize]
            data: [70_i8, 48_i8, 84_i8, 103_i8, 0_i8]
        };
        // let value = CString::new("F0Tg").unwrap();
        // key.data.clone_from_slice(value.as_bytes_with_nul());

        println!("Test 1 {:?}", key);

        let mut result_val = bindings::SMCVal_t::default();
        println!("Test 2 {:?}", result_val);

        let res2 = bindings::SMCReadKey2(&key, &mut result_val, conn);
        println!("Res2: {}", res2);

        println!("Test 3: {:?}", result_val);

        let res3 = bindings::SMCClose(conn);
        println!("Res3: {}", res3);

        bindings::init();
        bindings::finish();
    }
}