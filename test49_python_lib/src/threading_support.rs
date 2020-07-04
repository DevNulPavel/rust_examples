use pyo3::{
    // types::{
        //IntoPyDict
        // PyDict,
        // PyType
    // },
    prelude::*,
    wrap_pyfunction,
    //wrap_pymodule,
};
use rayon::{
    prelude::*
};

fn matches(word: &str, needle: &str) -> bool {
    let mut needle = needle.chars();
    for ch in word.chars().skip_while(|ch| !ch.is_alphabetic()) {
        match needle.next() {
            None => {
                return !ch.is_alphabetic();
            }
            Some(expect) => {
                if ch.to_lowercase().next() != Some(expect) {
                    return false;
                }
            }
        }
    }
    needle.next().is_none()
}

/// Count the occurences of needle in line, case insensitive
fn count_line(line: &str, needle: &str) -> usize {
    let mut total = 0;
    for word in line.split(' ') {
        if matches(word, needle) {
            total += 1;
        }
    }
    total
}

#[pyfunction]
pub fn search_using_threads(contents: &str, needle: &str) -> usize {
    contents
        .par_lines()
        .map(|line| {
            count_line(line, needle)
        })
        .sum()
}

pub fn export_to_python(_py: &Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(search_using_threads))?;
    
    Ok(())
}