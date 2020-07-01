use pyo3::{
    //types::{
        //IntoPyDict
    //},
    prelude::*,
    wrap_pyfunction,
    wrap_pymodule,
};

/// Функции надо помечать макросом
#[pyfunction]
fn sum_as_string_1(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn subfunction() -> String {
    "Subfunction".to_string()
}

#[pymodule]
fn submodule(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(subfunction))?;
    Ok(())
}

/// Имя функции модуля дожно быть таким же как у библиотеки и импортируемого модуля
#[pymodule]
fn rust_python_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    // Добавляем к нашему модулю функцию
    m.add_wrapped(wrap_pyfunction!(sum_as_string_1))?;

    // PyO3 знает функцию. Все наши Python интерфейсы должны быть описаны в отдельном модуле.
    // Заметим, что `#[pyfn()]` аннотация автоматически конвертирует аргументы из Python объектов
    // в Rust значения. Затем Rust возвращает значение назад в Python объект.
    // Аргумент _py отражает, что мы держим наш GIL в Python.
    #[pyfn(m, "sum_as_string_2")]
    fn sum_as_string_2_py(_py: Python, a: usize, b: usize) -> PyResult<String> {
        sum_as_string_1(a, b)
    }

    // Добавляем к нашему модулю подфункцию
    m.add_wrapped(wrap_pymodule!(submodule))?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::{
        types::{
            IntoPyDict
        },
    };
    

    #[test]
    fn test_module() {
        // TODO: Не хочет работать, надо скорее всего линковаться с какой-то библиотекой Python
        let gil = GILGuard::acquire();
        let py = gil.python();
        let supermodule = wrap_pymodule!(rust_python_lib)(py);
        let ctx = [("rust_python_lib", supermodule)].into_py_dict(py);
    
        py.run("assert rust_python_lib.submodule.subfunction() == 'Subfunction'", None, Some(&ctx)).unwrap();
    }
}