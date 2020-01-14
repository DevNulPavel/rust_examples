extern crate libc;

use std::ffi::CStr;
use std::os::raw::c_char;

// Инклюды из соседних файликов
use crate::expression::Expression;
use crate::parser;

// Перечисление типов, представление в виде u8 числа
//#[repr(u8)]
#[repr(C)]
pub enum ExpressionType {
    Add = 0,
    Subtract = 1,
    Multiply = 2,
    Divide = 3,
    UnaryMinus = 4,
    Value = 5,
}

// Выражение с двумя операндами
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PairOperands {
    left: *mut ExpressionFfi,
    right: *mut ExpressionFfi,
}

// Сишный UNION
#[repr(C)]
pub union ExpressionData {
    pair_operands: PairOperands,
    single_operand: *mut ExpressionFfi,
    value: i64,
}

// Структурка выражения
#[repr(C)]
pub struct ExpressionFfi {
    // Тип выражения
    expression_type: ExpressionType,
    // Данные
    data: ExpressionData,
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

// Реализацию методов для Expression можно делать в разных файликах
impl Expression {
    // Метод конвертации в C
    fn convert_to_c(&self) -> *mut ExpressionFfi {
        // Создаем данные для нашего выражения
        let expression_data = match self {
            Self::Value(value) => {
                ExpressionData { value: *value }
            },
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) | Self::Divide(left, right) => {
                ExpressionData {
                    pair_operands: PairOperands {
                        left: left.convert_to_c(),
                        right: right.convert_to_c()
                    }
                }
            },
            Self::UnaryMinus(operand) => ExpressionData {
                single_operand: operand.convert_to_c(),
            },
        };

        // Выражение
        let expression_ffi = match self {
            Self::Add(_, _) => ExpressionFfi {
                expression_type: ExpressionType::Add,
                data: expression_data,
            },
            Self::Subtract(_, _) => ExpressionFfi {
                expression_type: ExpressionType::Subtract,
                data: expression_data,
            },
            Self::Multiply(_, _) => ExpressionFfi {
                expression_type: ExpressionType::Multiply,
                data: expression_data,
            },
            Self::Divide(_, _) => ExpressionFfi {
                expression_type: ExpressionType::Multiply,
                data: expression_data,
            },
            Self::UnaryMinus(_) => ExpressionFfi {
                expression_type: ExpressionType::UnaryMinus,
                data: expression_data,
            },
            Self::Value(_) => ExpressionFfi {
                expression_type: ExpressionType::Value,
                data: expression_data,
            },
        };

        let expression_box = Box::new(expression_ffi);
        let raw_box_pointer = Box::into_raw(expression_box);
        return raw_box_pointer;
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

// Функция парсинга строчки в выражение
#[no_mangle]
pub extern "C" fn parse_arithmetic(s: *const c_char) -> *mut ExpressionFfi {
    unsafe {
        // Создаем Rust ссылочную строку из С-шной
        if let Ok(rust_string) = CStr::from_ptr(s).to_str() {
            if let Ok(parsed) = parser::parse(rust_string) {
                let data = parsed.convert_to_c();
                return data;
            }
        }
        return 0 as *mut ExpressionFfi;
    }
}

// Функция уничтожения выражения из памяти
#[no_mangle]
pub extern "C" fn destroy(expression: *mut ExpressionFfi) {
    // У указателей есть метод is_null
    if expression.is_null(){
        return;
    }

    // Обратно создаем умный указатель на кучу из сырого указателя
    let expr: Box<ExpressionFfi> = unsafe {
        Box::from_raw(expression)   // &mut *expression
    };

    match expr.expression_type {
        // Для типов из двух указателей - уничтожаем оба
        ExpressionType::Add | ExpressionType::Subtract | ExpressionType::Multiply | ExpressionType::Divide => {
            unsafe {
                destroy(expr.data.pair_operands.right);
                destroy(expr.data.pair_operands.left);
            }
            // ЗАтем уничтожаем само выражение в куче
            drop(expr);
        }
        // Для одинарного - уничтожаем выражение в одном указателе
        ExpressionType::UnaryMinus => {
            unsafe {
                destroy(expr.data.single_operand);
            }
            drop(expr);
        }
        // Для значения - уничтожаем само выражение
        ExpressionType::Value => {
            drop(expr);
        }
    };
}