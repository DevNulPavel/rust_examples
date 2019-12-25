// Таким образом можно указать для всего файла (#!) отключение неиспользуемых переменных
#![allow(unused_variables, unused_mut, dead_code)]

use std::ops::Deref;
use std::ops::DerefMut;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

// https://doc.rust-lang.ru/book/ch15-02-deref.html


fn box_test(){
    let b = Box::new(5);
    println!("b = {}", b);
}

////////////////////////////////////////////////////////////////////////////////////

struct Mp3 {
    audio: Vec<u8>,
    artist: Option<String>,
    title: Option<String>,
}

// Реализуем трейт Deref, чтобы получать доступ к данным напрямую из ссылки
impl Deref for Mp3 {
    type Target = Vec<u8>;
    // Вызывается при разыменовании данного объекта-указателя с помощью оператора *
    // *my_favorite_song преобразуется компилятором в *(my_favorite_song.deref())
    fn deref(&self) -> &Vec<u8> {
        &self.audio
    }
}

// Реализуем трейт DerefMut, чтобы получать доступ к ИЗМЕНЯЕМЫМ данным напрямую из ссылки
impl DerefMut for Mp3 {
    // То же самое, что обычный Deref, но возвращает изменяемую ссылку на содержимое
    // *my_favorite_song преобразуется компилятором в *(my_favorite_song.deref_mut())
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.audio
    }
}

fn test_deref_method(){
    let mut my_favorite_song = Mp3 {
        audio: vec![1, 2, 3],
        artist: Some(String::from("123")),
        title: Some(String::from("123")),
    };

    // Вызывается метод deref_mut неявно
    (*my_favorite_song).push(4); // Но вроде как не надо уже вызывать *
    assert_eq!(vec![1, 2, 3, 4], *my_favorite_song); // Вызывается метод deref
}

////////////////////////////////////////////////////////////////////////////////////

struct CustomSmartPointer {
    data: String,
}

// Данный типаж реализует деструктор, которыый вызывается, когда объект выходит из области видимости
impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer!");
    }
}

fn test_drop_method(){
    let c = CustomSmartPointer { 
        data: String::from("some data") 
    };
    println!("CustomSmartPointer created.");
    println!("Wait for it...");
}

////////////////////////////////////////////////////////////////////////////////////

enum List {
    Cons(i32, Rc<List>),
    Nil,
}

fn test_reference_counter(){
    // Создаем новый список элементов с вложенностью
    let a = Rc::new(List::Cons(5, Rc::new(List::Cons(10, Rc::new(List::Nil)))));
    let b = List::Cons(3, a.clone());
    let c = List::Cons(4, a.clone());

    let a = Rc::new(List::Cons(5, Rc::new(List::Cons(10, Rc::new(List::Nil)))));
    println!("rc = {}", Rc::strong_count(&a));

    let b = List::Cons(3, a.clone());
    println!("rc after creating b = {}", Rc::strong_count(&a));
    {
        let c = List::Cons(4, a.clone());
        println!("rc after creating c = {}", Rc::strong_count(&a));
    }
    println!("rc after c goes out of scope = {}", Rc::strong_count(&a));
}

////////////////////////////////////////////////////////////////////////////////////

fn a_fn_that_immutably_borrows(a: &i32) {
    println!("a is {}", a);
}

fn a_fn_that_mutably_borrows(b: &mut i32) {
    *b += 1;
}

fn demo(r: &RefCell<i32>) {
    a_fn_that_immutably_borrows(&r.borrow());
    a_fn_that_mutably_borrows(&mut r.borrow_mut());
    a_fn_that_immutably_borrows(&r.borrow());
}

fn test_ref_cell(){
    let data = RefCell::new(5);
    demo(&data);
}

////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn test_weak_ref(){
    // Создаем умный указатель на будущий чилд-нод (Rc == shared_ptr)
    let leaf = Rc::new(Node {
        value: 3,
        parent: RefCell::new(Weak::new()), // Родителя пока нету, поэтому слабая ссылка (Weak == weak_ptr)
        children: RefCell::new(vec![]),    // Детей тоже нету
    });

    println!(
        "Leaf: strong = {}, weak = {}",
        Rc::strong_count(&leaf), // Так можно молучить количество обычных ссылок
        Rc::weak_count(&leaf),   // Так можно получить количество слабых ссылок
    );

    {
        // Создаем теперь новый корневой нод
        let root = Rc::new(Node {
            value: 5,
            parent: RefCell::new(Weak::new()), // Родителей у корня нету
            children: RefCell::new(vec![leaf.clone()]), // Чилдом будет как раз тот узел выше
        });

        // Прописываем узлу ссылку на новый корень в качестве родителя
        let root_weak_ptr = Rc::downgrade(&root); // Получаем слабую ссылку из Rc (weak_ptr из shared_ptr)
        *(leaf.parent.borrow_mut()) = root_weak_ptr;

        println!(
            "Root: strong = {}, weak = {}",
            Rc::strong_count(&root),
            Rc::weak_count(&root),
        );

        println!(
            "Leaf: strong = {}, weak = {}",
            Rc::strong_count(&leaf),
            Rc::weak_count(&leaf),
        );

        println!("Root destroy");
    }

    println!(
        "Leaf: parent = {:?}", 
        leaf.parent.borrow().upgrade()
    );
    println!(
        "Leaf: strong = {}, weak = {}",
        Rc::strong_count(&leaf),
        Rc::weak_count(&leaf),
    );
}

fn main() {
    //test_deref_method();
    //test_drop_method();
    //test_ref_cell();
    test_weak_ref();

    /*let mut num = 5 as i32;
    let mut str_val = String::new();
    {
        let y = &mut num;
        *y += 1; // Для изменения содержимого надо использовать разыменование, так как число не реализует Deref
        println!("{}", y);

        let z = &mut str_val;
        (*z).push_str("asd"); // Можно выполнять разыменование для ссылки
        z.push_str("asdsd"); // Однако строка реализует типаж Deref, позволяя работать с содержимым напрямую
        println!("{}", z);
    }*/
}