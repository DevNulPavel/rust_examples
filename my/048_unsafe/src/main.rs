
fn get_pair_mut<T>(xs: &mut [T], 
                   index1: usize, 
                   index2: usize) -> (&mut T, &mut T){
    let ptr1: *mut T = &mut xs[index1];
    let ptr2: *mut T = &mut xs[index2];

    assert!(ptr1 != ptr2);

    unsafe {
        (&mut *ptr1, &mut *ptr2)
    }
}

fn main(){
    let mut test_array = [1, 2, 3, 4];
    println!("{:?}", test_array);
    let (v1, v2) = get_pair_mut(&mut test_array, 1, 3);
    *v1 = 10;
    *v2 = 20;
    println!("{:?}", test_array);
}