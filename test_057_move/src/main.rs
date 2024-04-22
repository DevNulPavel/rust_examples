struct TestStruct{
    a: i32,
    b: i32,
    c: Box<i32>
}

// При вызове функции происходит перемещение, аддресс новой переменной внутри новый
// Точно так же адреса переменных внутри тоже новые
// Но при использовании Box/Arc и тд, указатель внутри не должен мееняться

fn test1(val: TestStruct){
    let pointer: *const TestStruct = &val;
    let filed_pointer: *const i32 = &val.a;
    let box_filed_pointer: *const i32 = val.c.as_ref();
    println!("New address: {}", pointer as usize);
    println!("New field address: {}", filed_pointer as usize);
    println!("New box field address: {}", box_filed_pointer as usize);
}

#[inline]
fn test2(val: TestStruct){
    let pointer: *const TestStruct = &val;
    let filed_pointer: *const i32 = &val.a;
    let box_filed_pointer: *const i32 = val.c.as_ref();
    println!("New address: {}", pointer as usize);
    println!("New field address: {}", filed_pointer as usize);
    println!("New box field address: {}", box_filed_pointer as usize);
}

fn main() {
    let val = TestStruct{
        a: 10,
        b: 20,
        c: Box::new(30)
    };

    let pointer: *const TestStruct = &val;
    let filed_pointer: *const i32 = &val.a;
    let box_filed_pointer: *const i32 = val.c.as_ref();
    println!("Source address: 0x{:0x}", pointer as usize);
    println!("Source field address: {}", filed_pointer as usize);
    println!("Source box field address: {}", box_filed_pointer as usize);

    // test1(val);
    test2(val);
}
