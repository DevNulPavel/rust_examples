const addon = require('../native');


async function main(){
    let text = addon.hello();
    console.log(`Text from Rust: ${text}`);

    let number = addon.number_function();
    console.log(`Number from Rust: ${number}`);

    let array = addon.make_an_array();
    console.log(`Array from Rust: ${array}`);

    let args_len = addon.get_args_len("test", 123);
    console.log(`Args count: ${args_len}`);

    //////////////////////////////////////////////////////////////////////////////////////////

    let object = addon.test_objects();
    let object_func = object.function_property;
    let func_result = object_func();
    console.log(`Object: ${object}`);
    console.log(`Function: ${object_func}, returnned from function "${func_result}"`);

    //////////////////////////////////////////////////////////////////////////////////////////

    let modify_func = addon.modify_object_this; // Копируем функцию, чтобы this использовался из контекста, а не из addon
    let func1 = function (){
        // this - это контекст данной функции
        //this.modified = false;
        modify_func();
        console.log(`This modify (func): ${this.modified}`);
    };
    func1();
    let func2 = () => {
        // this - это контекст функции main
        //this.modified = false;
        modify_func();
        console.log(`This modify (lambda): ${this.modified}`);
    };
    func2();

    //////////////////////////////////////////////////////////////////////////////////////////

    addon.function_as_parameter((param)=>{
        console.log(`Js function called from Rust with param: ${param}`);
        return 10;
    });

    //////////////////////////////////////////////////////////////////////////////////////////

    //const date = new Date('December 31, 1975, 23:15:30 GMT+11:00');
    let year = addon.construct_js_function(Date);
    console.log(`Received year: ${year}`);
    // construct_obj.getUTCFullYear = (param)=>{
    //     console.log(`Js function called from Rust with param: ${param}`);
    // };
    //construct_obj(12);
    /*addon.construct_js_function((param)=>{
        console.log(`Js function called from Rust with param: ${param}`);
        return 10;
    })*/

    //////////////////////////////////////////////////////////////////////////////////////////

    const john = new addon.Employee(10, "John");
    john.name(); // John
    john.greet(); // Hi John!
    john.askQuestion(); // How are you?

    //////////////////////////////////////////////////////////////////////////////////////////

    // Итерируемся много много раз в фоновом потоке
    addon.perform_async_task(2.0, (err, res) => {
        console.log("Background task finished: sleep duration = ", res);
    });
    try{
        addon.perform_async_task(-2.0, (err, res) => {
            console.log(err);
        });
    }catch(err){
        console.log(err);
    }
    addon.perform_async_task(15.0, (err, res) => {
        console.log(err);
    });
    const async_task_promise = new Promise((resolve, reject) => {
        addon.perform_async_task(3.0, (err, res) => {
            console.log("Background task finished from promise: sleep duration = ", res);
            if (!err) {
                resolve(res);
            }else{
                reject(err);
            }
        });
    });
    console.log("Main thread finished");
    async_task_promise.await;
}

main();