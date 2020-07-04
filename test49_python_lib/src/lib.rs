mod threading_support;

use pyo3::{
    types::{
        //IntoPyDict
        PyDict,
        PyType
    },
    prelude::*,
    wrap_pyfunction,
    wrap_pymodule,
};

/// Функции надо помечать макросом
#[pyfunction]
fn sum_as_string_1(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// Можно описывать список параметров функции для процедурного макроса
#[pyfunction(kwds="**")]
fn num_kwds(kwds: Option<&PyDict>) -> usize {
    kwds.map_or(0, |dict| dict.len())
}

/// Функция сложения значений, позволяет указаывать имена переменных, / - значит конец позиционных параметров
#[pyfunction]
#[text_signature = "(a, b, /)"]
fn add(a: u64, b: u64) -> u64 {
    a + b
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

////////////////////////////////////////////////////////////

#[pyclass]
#[text_signature = "(c, d, /)"]
struct MyClass {
    #[pyo3(get)]
    val: i32,
    _text: String
}

#[pymethods]
impl MyClass {
    // the signature for the constructor is attached
    // to the struct definition instead.
    #[new]
    fn new(c: i32, d: &str) -> Self {
        Self {
            val: c,
            _text: d.to_string()
        }
    }

    // the self argument should be written $self
    #[text_signature = "($self, e, f)"]
    fn my_method(&self, e: i32, f: i32) -> i32 {
        e + f + self.val
    }

    #[classmethod]
    #[text_signature = "(cls, e, f)"]
    fn my_class_method(_cls: &PyType, e: i32, f: i32) -> i32 {
        e + f
    }

    #[staticmethod]
    #[text_signature = "(e, f)"]
    fn my_static_method(e: i32, f: i32) -> i32 {
        e + f
    }
}

/////////////////////////////////////////////////////////////

fn test_py_cell(){
    // Тестирование аналога RefCell - PyCell
    let gil = Python::acquire_gil();
    let py = gil.python();
    let obj = {
        let my_class_obj = MyClass{ 
            val: 3, 
            _text: "Test".to_string() 
        };
        PyCell::new(py, my_class_obj).unwrap()
    };
    {
        // Получаем ссылку на наш объект
        let obj_ref = obj.borrow(); // Get PyRef
        assert_eq!(obj_ref.val, 3);

        // Нельзя получить PyRefMut до тех пор пока все PyRefs не уничтожены
        assert!(obj.try_borrow_mut().is_err());
    }

    {
        // Получаем PyRefMut
        let mut obj_mut = obj.borrow_mut();
        obj_mut.val = 5;

        // Нельзя в рантайме получить никакие больше PyRefMut, пока есть хотя бы одна ссылка на него
        assert!(obj.try_borrow().is_err());
        assert!(obj.try_borrow_mut().is_err());
    }
    
    // You can convert `&PyCell` to a Python object
    pyo3::py_run!(py, obj, "assert obj.val == 5");
}

/// Тестирование долгоживущего объекта
fn test_python_longlife_obj(){
    // Получаем блокировку Python
    let gil = Python::acquire_gil();
    // Получаем контекст
    let py = gil.python();
    let obj = {
        let obj = MyClass{ 
            val: 1, 
            _text: "Test".to_string() 
        };
        Py::new(py, obj)
            .unwrap()
    };

    // Получаем ссылку &PyCell
    let cell = obj.as_ref(gil.python()); // AsPyRef::as_ref returns &PyCell
    // Получаем PyRef<T> объект
    let obj_ref = cell.borrow(); // Get PyRef<T>

    assert_eq!(obj_ref.val, 1);
}

/////////////////////////////////////////////////////////////

/// Имя функции модуля дожно быть таким же как у библиотеки и импортируемого модуля
#[pymodule]
fn rust_python_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    // Добавляем к нашему модулю функцию
    m.add_wrapped(wrap_pyfunction!(sum_as_string_1))?;
    m.add_wrapped(wrap_pyfunction!(num_kwds))?;
    m.add_wrapped(wrap_pyfunction!(add))?;

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

    // Так мы добавляем конкретный класс к модулю
    m.add_class::<MyClass>()?;

    ////////////////////////////////////////////////

    // Тестирование аналога RefCell - PyCell
    test_py_cell();

    // Тест долгоживущего объекта
    test_python_longlife_obj();
    

    // Экспоритрование всех наших данных
    threading_support::export_to_python(&_py, m)?;

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