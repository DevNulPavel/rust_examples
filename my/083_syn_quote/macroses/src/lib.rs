#![feature(proc_macro_diagnostic)]

// use proc_macro2::Span;
// Вспомогательная стандартная библиотека для работы с макросами
// use proc_macro::TokenStream as TokenStream1;
// Библиотека внешняя более удобная для работы с макросами
// use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Expr, Ident, Token, Type, Visibility,
};

struct LazyStatic {
    /// Режим видимости
    /// pub, pub(crate), pub(super)
    visibility: Visibility,
    // Имя переменной
    var_name: Ident,
    // Тип переменной
    var_type: Type,
    // Выражение, которым мы инициализируем нашу переменную
    initial_expression: Expr,
}

/// Реализуем парсинг выражения для получения структуры LazyStatic с информацией
impl Parse for LazyStatic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Сначала в потоке токенов пытаемся распарсить токен видимости
        // Visibility тоже реализует аналогичный метод parse
        // Мы можем вызвать парсинг в том числе вот так: input.call(Visibility::parse)?;
        let visibility: Visibility = input.parse()?;

        // Дальше мы ожидаем токены static + ref
        input.parse::<Token!(static)>()?;
        input.parse::<Token!(ref)>()?;

        // Затем мы читаем имя нашей переменной
        let var_name: Ident = input.parse()?;

        // Читаем после этого символ `:`
        input.parse::<Token!(:)>()?;

        // Затем мы читаем тип нашей переменной
        let var_type: Type = input.parse()?;

        // Читаем после этого символ `=`
        input.parse::<Token!(=)>()?;

        // Затем мы читаем тип нашей переменной
        let initial_expression: Expr = input.parse()?;

        // Читаем после этого символ `;`
        input.parse::<Token!(;)>()?;

        // proc_macro_error::emit_error!(input.span(), "token");
        // emit_call_site_error!("Test");
        // return Err(input.error("Invalid token"));

        Ok(LazyStatic {
            visibility,
            var_name,
            var_type,
            initial_expression,
        })
    }
}

fn check_variable_name(var_name: &Ident) -> Result<(), ()> {
    // Можно таким вот образом выводить предупреждения
    // с помощью стандартного Span из proc_macro библиотеки
    // #[cfg(proc_macro_diagnostic)]

    if var_name == "WARNING_TEST_NATIVE" {
        // Сначала мы получаем span версии 1
        let span_v2: proc_macro2::Span = var_name.span();
        let span_v1: proc_macro::Span = span_v2.unwrap();
        // Затем мы можем сгенерировать в данном месте ошибку, но ошибка не ломает дальше парсинг
        span_v1
            .warning("This is the warning about var name using native span warnings")
            .emit();
        Err(())
    } else if var_name == "ERROR_TEST_NATIVE" {
        let span_v2: proc_macro2::Span = var_name.span();
        let span_v1: proc_macro::Span = span_v2.unwrap();
        span_v1
            .error("This is the error about var name using native span warnings")
            .emit();

        Err(())
    } else if var_name == "WARNING_TEST_LIB" {
        // Описание синтаксиса
        // https://docs.rs/proc-macro-error/1.0.4/proc_macro_error/#note-attachments
        emit_warning!(
            var_name,
            "Testing variable name";
            help = "My help";
            note = "Test note";
            yay = "Test yay";
            wow = var_name.span() => "custom span";
        );
        Err(())
    } else if var_name == "ERROR_TEST_LIB" {
        emit_error!(var_name, "Testing variable name");
        Err(())
    } else {
        Ok(())
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn lazy_static(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Сначала парсим входные токены в структуру данных, описывающую функцию
    let LazyStatic {
        visibility,
        var_name,
        var_type,
        initial_expression,
    } = parse_macro_input!(input as LazyStatic);

    // Вызываем проверку переменной
    if check_variable_name(&var_name).is_err() {
        // Вырубаем дальнейший анализ, возвращая пустой поток токенов
        return proc_macro::TokenStream::new();
    }

    // Проверяем, что у нас выражение не пустое
    if let Expr::Tuple(ref init) = initial_expression {
        if init.elems.is_empty() {
            initial_expression
                .span()
                .unwrap()
                .error("I can't think of a legitimate use for lazily initializing the value `()`")
                .emit();

            // Вырубаем дальнейший анализ, возвращая пустой поток токенов
            return proc_macro::TokenStream::new();
        }
    }

    // Ассерт, заставляющий тип реализовать Sync.
    // Указываем на ошибку для нужного поля с помощью передачи Span от выбранной переменной.
    // Если нет, то пользователь увидит сообщение об ошибке вида:
    //
    //     error[E0277]: the trait bound `*const (): std::marker::Sync` is not satisfied
    //       --> src/main.rs:10:21
    //        |
    //     10 |     static ref PTR: *const () = &();
    //        |                     ^^^^^^^^^ `*const ()` cannot be shared between threads safely
    //
    // Делается такая проверка за счет попытки скомпилировть фейковую структуру, где тип
    // должен реализовать трейт Sync
    let assert_sync = quote_spanned! {var_type.span()=>
        struct _AssertSync where #var_type: std::marker::Sync;
    };

    // Точно такая же проверка, но уже для Sized
    //
    //     error[E0277]: the trait bound `str: std::marker::Sized` is not satisfied
    //       --> src/main.rs:10:19
    //        |
    //     10 |     static ref A: str = "";
    //        |                   ^^^ `str` does not have a constant size known at compile-time
    let assert_sized = quote_spanned! {var_type.span()=>
        struct _AssertSized where #var_type: std::marker::Sized;
    };

    // Оборачиваем наше выражение
    let init_ptr = quote_spanned! {initial_expression.span()=>
        // Сначала мы перемещаем в кучу результат нашего выражения, а после этого
        // мы получаем уже сырой указатель на кучу
        Box::into_raw(Box::new(#initial_expression))
    };

    // Формируем уже конечное выражение
    let expanded = quote! {
        // Объявляем структуру с нашим именем и настройкой видимости
        #[allow(non_camel_case_types)]
        #[allow(upper_case_types)]
        #[warn(snake_case_types)]
        #[warn(camel_case_types)]
        #visibility struct #var_name;

        // Реализуем Deref для нашей переменной
        impl std::ops::Deref for #var_name {
            // Возвращаемым типом будет как раз наш тип
            type Target = #var_type;

            fn deref(&self) -> &#var_type {
                // Добавляем ограничения Sync/Sized внутри нашей функции
                #assert_sync
                #assert_sized

                // Создаем Once переменную + нулевой указатель на тип
                static ONCE: std::sync::Once = std::sync::Once::new();
                static mut VALUE: *mut #var_type = 0 as *mut #var_type;

                unsafe {
                    // Инициализируем синхронно эту нашу переменную лишь один раз
                    ONCE.call_once(|| VALUE = #init_ptr);
                    // Затем просто возвращаем ссылку на наш разыменованный указатель на данные
                    &*VALUE
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
