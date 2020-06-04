use neon::prelude::*;
struct BackgroundTask{
    // Входное значение
    sleep_seconds: u64
}

impl Task for BackgroundTask {
    // Выход задачи
    type Output = u64;
    // Тип ошибки
    type Error = String;
    // Выдаваемое значение в виде результата
    type JsEvent = JsNumber;

    // Данный метод выполняется в пуле потоков libuv в NodeJs
    fn perform(&self) -> Result<Self::Output, Self::Error> {
        if self.sleep_seconds < 10{
            std::thread::sleep(std::time::Duration::from_secs(self.sleep_seconds));
            Ok(self.sleep_seconds)    
        }else{
            Err("Sleep duration must be less than 10 sec".to_owned())
        }
    }

    // Данный метод вызывается уже в главном потоке после завершения работы
    fn complete(self, 
                mut cx: TaskContext, 
                result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {

        match result{
            Ok(val) => Ok(cx.number(val as f64)),
            Err(e) => cx.throw_error(e),
        }
    }
}

pub fn perform_async_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let sleep_seconds = cx.argument::<JsNumber>(0)?.value();
    
    // Проверяем валидность значения
    if sleep_seconds.is_sign_negative(){
        return cx.throw_error("Sleep duration must be non-zero value");
    }

    // Получаем коллбек, функция должна быть формата "function callback(err, value)"
    // Первым параметром идет ошибка, вторым - результат, это общее соглашение в JS
    let func = cx.argument::<JsFunction>(1)?;

    let task = BackgroundTask{
        sleep_seconds: sleep_seconds as u64
    };
    
    task.schedule(func);

    Ok(cx.undefined())
}
