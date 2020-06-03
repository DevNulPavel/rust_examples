var addon = require('../native');

function main(){
    let text = addon.hello();
    console.log(`Text from Rust: ${text}`);

    let number = addon.number_function();
    console.log(`Number from Rust: ${number}`);

    let array = addon.make_an_array();
    console.log(`Array from Rust: ${array}`);

    let args_len = addon.get_args_len("test", 123);
    console.log(`Args count: ${args_len}`);

    let object = addon.test_objects();
    let object_func = object.function_property;
    let func_result = object_func();
    console.log(`Object: ${object}`);
    console.log(`Function: ${object_func}, returnned from function "${func_result}"`);
}

main();