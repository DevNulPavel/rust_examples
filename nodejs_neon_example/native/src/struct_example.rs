use neon::{
    prelude::{
        *
    },
    borrow::{
        Ref,
        //RefMut
    }
};

pub struct Employee {
    // Переменные структуры мапятся в свойства класса JS 
    pub _id: i32,
    pub name: String
}

impl Employee {
    pub fn new(id: i32, name: String) -> Self { 
        Self{ 
            _id: id, 
            name 
        } 
    }

    /*fn talk(mut cx: FunctionContext) -> JsResult<JsValue> {
        let obj = cx.string("How are you doing?")
            .upcast();
        Ok(obj)
    }*/
}

declare_types! {
    /// Описываем класс, который оборачивает Employee записи
    pub class JsEmployee for Employee {
        // Метод-конструктор JS
        init(mut cx) {
            // Получаем id
            let id = cx.argument::<JsNumber>(0)?
                .value();
            
            // Получаем имя
            let name: String = cx
                .argument::<JsString>(1)?
                .value();

            let e = Employee::new(id as i32, name);

            Ok(e)
        }

        method talk(mut cx) {
            Ok(cx.string("How are you doing?").upcast())
        }

        method name(mut cx) {
            // Получаем текущий объект
            let this = cx.this();
            // Получаем имя
            let name: String = {
                // Получаем блокировку контекста
                let guard = cx.lock();
                // Получаем мутабельную ссылку на наш объект
                let rust_obj: Ref<&mut Employee> = this.borrow(&guard);
                // Создаем копию имени
                rust_obj.name.clone()
            };
            println!("RUST: {}", &name);
            Ok(cx.undefined().upcast())
        }

        method greet(mut cx) {
            // Получаем текущий объект
            let this = cx.this();
            let msg: String = {
                // Получаем блокировку контекста
                let guard = cx.lock();
                // Получаем мутабельную ссылку на наш объект
                let greeter: Ref<&mut Employee> = this.borrow(&guard);
                // Форматируем строку
                format!("RUST: Hi {}!", greeter.name)
            };
            println!("{}", &msg);
            Ok(cx.string(&msg).upcast())
        }

        method askQuestion(mut cx) {
            println!("RUST: {}", "How are you?");
            Ok(cx.undefined().upcast())
        }
    }
}

// register_module!(mut cx, { 
//     Ok(())
// });