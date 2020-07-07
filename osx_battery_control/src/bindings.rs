// #[link(name = "testlib")]
extern {
    pub fn init();
    pub fn finish();
}

// #[repr(C)]
// #[derive(Default, Debug)]
// pub struct UInt32Char_t{
//     pub data: [std::os::raw::c_char; 5_usize]
// }

// #[derive(Default, Debug)]
// #[repr(C)]
// pub struct SMCBytes_t{
//     pub data: [std::os::raw::c_uchar; 32_usize]
// }

// // UInt32Char_t            key;
// // UInt32                  dataSize;
// // UInt32Char_t            dataType;
// // SMCBytes_t              bytes;

// #[derive(Default, Debug)]
// #[repr(C)]
// pub struct SMCVal_t{
//     pub key: UInt32Char_t,
//     pub data_size: u32,
//     pub data_type: UInt32Char_t,
//     pub bytes: SMCBytes_t
// }

// extern "C" {
//     pub fn SMCOpen(conn: *mut IOKit_sys::io_connect_t) -> mach::kern_return::kern_return_t;
//     pub fn SMCClose(conn: IOKit_sys::io_connect_t) -> mach::kern_return::kern_return_t;

//     // kern_return_t SMCReadKey2(UInt32Char_t key, SMCVal_t *val,io_connect_t conn);
//     pub fn SMCReadKey2(key: *const UInt32Char_t, val: *mut SMCVal_t, conn: IOKit_sys::io_connect_t) -> mach::kern_return::kern_return_t;
// }