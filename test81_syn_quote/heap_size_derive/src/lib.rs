// Вспомогательная стандартная библиотека для работы с макросами
use proc_macro::TokenStream as TokenStream1;
// Библиотека внешняя более удобная для работы с макросами
use proc_macro2::TokenStream as TokenStream2;
// Специальный макрос для генерации токенов Rust на основе шаблона
use quote::{quote, quote_spanned};
// Парсинг ввода в виде макроса + тип
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, Generics, Index,
    TypeParamBound,
};

/// Добавляем для всех дочерних Generic параметрв ограничение в T: HeapSize
fn add_trait_bounds(mut generics: Generics, bound: TypeParamBound) -> Generics {
    // Мутабельно обходим все generic параметры нашей структуры со ссылками на объект
    /*for param in generics.params.iter_mut() {
        // Если параметр является конкретным шаблонным типом
        match param {
            GenericParam::Type(type_param) => {
                //  Добавляем для данного типа конкретное ограничение
                type_param.bounds.push(bound);
            }
            _ => {}
        }
    }*/

    // Для упорощения можно использовать итератор generics.type_params_mut()
    for type_param in generics.type_params_mut() {
        //  Добавляем для данного типа конкретное ограничение
        type_param.bounds.push(bound.clone());
    }
    generics
}

/// Создаем выражение для суммарного вычисления размера в куче для каждого поля нашей структуры
fn heap_size_sum(data: &Data) -> TokenStream2 {
    // Сначала определим к чему применяется наш макрос
    // Это структура, enum, union?
    match data {
        // Если это структурка
        Data::Struct(data) => {
            // Поля внутри нашей структурки тоже могут быть разными
            match &data.fields {
                // Если это именованные поля
                Fields::Named(fields) => {
                    // Разворачиваем тогда все поля в выражение в духе
                    //      0 + self.x.heap_size() + self.y.heap_size() + self.z.heap_size()
                    // однако используя при этом вызов функции с параметром вместо вызова через точку
                    // при этом указывая полный путь к вызываемой функции: heapsize::HeapSize::heap_size_of_children
                    //
                    // Мы заботимся об использовании span каждого поля `syn::Field`
                    // Это нужно для того, чтобы в случае, если один из полей не реализует
                    // компилятор мог конкретно сказать какое это поле

                    // Обходим каждое поле в структуре
                    let recurse = fields.named.iter().map(|f| {
                        // Получаем имя нашего поля
                        let name = &f.ident;
                        // Получаем месторасположение нашего поля в коде
                        let span = f.span();
                        // Генерируем набор токенов вместе со span данного поля
                        quote_spanned! {span=>
                            heap_size::HeapSize::heap_size_of_children(&self.#name)
                        }
                    });
                    // Затем генерируем код, который добавляет все наши поля к коду в виде суммирования
                    quote! {
                        0 #(+ #recurse)*
                    }
                }
                // Если у нас структура без именованных полей
                Fields::Unnamed(fields) => {
                    // Разворачиваем в выражение в духе
                    //     0 + heap_size::HeapSize::heap_size_of_children(&self.0) + ...

                    // Идем по всем неименованным полям
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = Index::from(i);
                        let span = f.span();
                        quote_spanned! {span=>
                            heap_size::HeapSize::heap_size_of_children(&self.#index)
                        }
                    });
                    quote! {
                        0 #(+ #recurse)*
                    }
                }
                // Если поле - это пустой тип ()
                Fields::Unit => {
                    // Unit structs cannot own more than 0 bytes of heap memory.
                    quote!(0)
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Здесь мы описываем тип нашего Derive макроса и его название
#[proc_macro_derive(HeapSize)]
pub fn derive_heap_size(input: TokenStream1) -> TokenStream1 {
    // Парсим входные токены в синтаксическое дерево интерпретируя ввод как DeriveInput
    // Формат который мы хотим получить как раз и указывается после as
    // То есть это не каст, а параметр для макроса, который фейлится при ошибке
    let input = parse_macro_input!(input as DeriveInput);

    // Получаем имя структуры к которой применяется наш макрос
    let name = input.ident;
    println!("Input ident: {:?}", name);

    // Добавляем для всех дочерних Generic параметрв ограничение в T: HeapSize
    // parse_quote вычисляет тип по использованию переменной после, нужен для
    // генерации из текста конкретных токенов
    let generic_limits = parse_quote!(heap_size::HeapSize);
    let generics = add_trait_bounds(input.generics, generic_limits);

    // Разворачиваем дженерики на составляющие
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Создаем выражение для суммарного вычисления размера в куче для каждого поля нашей структуры
    // .data - это содержимое нашей структурки
    let sum = heap_size_sum(&input.data);

    // Теперь генерируем уже нашу реализацию для трейта
    let expanded = quote! {
        // The generated impl.
        impl #impl_generics heap_size::HeapSize for #name #ty_generics #where_clause {
            fn heap_size_of_children(&self) -> usize {
                #sum
            }
        }
    };

    // Теперь снова делаем набор токенов для компилятора
    proc_macro::TokenStream::from(expanded)
}
