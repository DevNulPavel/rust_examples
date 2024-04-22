
fn partition(v: &mut [i32]) -> usize {
    // Опорная точка - конец
    let pivot = v.len() - 1;
    let mut i = 0;
    // Итерируемся от 0 до конца
    for j in 0..pivot {
        // Если текущее значение меньше конечного
        if v[j] <= v[pivot] {
            // Тогда меняем значение первого элемента и текущего
            v.swap(i, j);
            i += 1;
        }
    }
    v.swap(i, pivot);
    i
}

pub fn quick_sort(arr: &mut[i32]) {
    if arr.len() <= 1 {
        return;
    }

    let mid = partition(arr);
    let (lo, hi) = arr.split_at_mut(mid);
    rayon::join(|| quick_sort(lo), || quick_sort(hi));
}