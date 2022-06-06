use quote::ToTokens;
use std::collections::HashSet;
use syn::{
    fold::{self, Fold},
    parse::{Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
    Expr, Ident, Local, Pat, Stmt, Token,
};

/// Парсим список переменных, разделенных с помощью запятой
///
///     a, b, c
///
/// Таким образом компилятор парсит аргументы в наши аттрибуты,
/// это то, что находится внутри скобок
///
///     #[trace_var(a, b, c)]
///                 ^^^^^^^
pub struct Args {
    // HashSet используем для проверки уникальности
    vars: HashSet<Ident>,
}

// Реализация трейта парсинга из библиотеки syn
impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        // panic!("Arguments parsing input: {}", input);

        // Punktuated - это последовательность каких-либо символов, разделенных определенным знаком пунктуации
        // std::option::Option
        // 1 + 2 + 3
        // Конкретно здесь парсим аргументы, разделенные символом ","
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}

impl Args {
    /// Определяем, что данное выражение есть в аргументах и его нужно выводить.
    /// Переменная находится слева от оператора равно
    fn should_print_expr(&self, e: &Expr) -> bool {
        match *e {
            // Тип вырежения - это какая-то переменная-путь
            Expr::Path(ref e) => {
                // Если путь начинается с :  и длина не равна единице этого пути - значит не то
                if e.path.leading_colon.is_some() || e.path.segments.len() != 1 {
                    false
                } else {
                    // Получаем в пути первую переменную
                    let first = e.path.segments.first().unwrap();
                    // Эта переменная есть у нас в списке отладки и у нее нет аргументов больше?
                    self.vars.contains(&first.ident) && first.arguments.is_empty()
                }
            }
            _ => false,
        }
    }

    /// Определяем, что данный паттерн - это идентификатор, равный ключу одной из переменных
    fn should_print_pat(&self, p: &Pat) -> bool {
        match p {
            Pat::Ident(ref p) => self.vars.contains(&p.ident),
            _ => false,
        }
    }

    /// Создает выражение, которое присваивает правую сторону левой стороне выражения и печатает значение одновременно
    ///
    ///     // Before
    ///     VAR = INIT
    ///
    ///     // After
    ///     { VAR = INIT; println!("VAR = {:?}", VAR); }
    fn assign_and_print(&mut self, left: Expr, op: &dyn ToTokens, right: Expr) -> Expr {
        // Правое выражение дальше рекурсивно разбираем, вдруг там тоже есть что обработать
        let right = fold::fold_expr(self, right);
        // Затем формируем выражение и автоматически приводим его к типу Expr
        parse_quote!({
            #left #op #right;
            println!(concat!("DBG: ", stringify!(#left), " = {:?}"), #left);
        })
    }

    /// Создает новое выражение, которое присваивает левой стороне, правую + печатает значение
    ///
    ///     // Before
    ///     let VAR = INIT;
    ///
    ///     // After
    ///     let VAR = { let VAR = INIT; println!("VAR = {:?}", VAR); VAR };
    fn let_and_print(&mut self, local: Local) -> Stmt {
        let Local { pat, init, .. } = local;
        let init = self.fold_expr(*init.unwrap().1);
        let ident = match pat {
            Pat::Ident(ref p) => &p.ident,
            _ => unreachable!(),
        };
        parse_quote! {
            let #pat = {
                #[allow(unused_mut)]
                let #pat = #init;
                println!(concat!("DBG: ", stringify!(#ident), " = {:?}"), #ident);
                #ident
            };
        }
    }
}

/// Трейт Fold - это способ обходить дерево синтаксиса и заменять там определенные ноды.
///
/// Syn предоставляет еще два разных синтаксиса для обхода дерева:
/// - `Visit` обходит заимствованное синтаксическое дерево
/// - `VisitMut` - обходит синтаксическое дерево в мутабельном стиле
///
/// Все три трейта имеют метод, соответствующий каждому типу нода в Syn дереве.
/// Все эти методы имеют стандартные реализации, которые просто рекурсивно обходят ноды.
/// Мы можем перегрузить лишь те методы, для которых мы хотим нестандартное поведение.
/// В данном случае мы хотим изменять `Expr` and `Stmt` ноды
impl Fold for Args {
    // Метод, модифицирующий выражения
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            // Выражение присваивания в духе: a = func()
            Expr::Assign(e) => {
                // Определяем, нужно ли нам печатать выражение для левой стороны присваивания
                if self.should_print_expr(&e.left) {
                    // Формируем новое выражение с распечатыванием
                    self.assign_and_print(*e.left, &e.eq_token, *e.right)
                } else {
                    // Выполняем дальше рекурсивный разбор, может быть внутри есть что-то
                    Expr::Assign(fold::fold_expr_assign(self, e))
                }
            }
            // Операция в духе +=
            Expr::AssignOp(e) => {
                // Определяем, надо ли нам левую сторону как-то обрабатывать?
                if self.should_print_expr(&e.left) {
                    self.assign_and_print(*e.left, &e.op, *e.right)
                } else {
                    // Если не надо, тогда рекурсивно обходим выражение дальше, вдруг внутри есть что-то
                    Expr::AssignOp(fold::fold_expr_assign_op(self, e))
                }
            }
            _ => fold::fold_expr(self, e),
        }
    }

    // Модификация выражения присваивания
    fn fold_stmt(&mut self, s: Stmt) -> Stmt {
        match s {
            Stmt::Local(s) => {
                if s.init.is_some() && self.should_print_pat(&s.pat) {
                    self.let_and_print(s)
                } else {
                    Stmt::Local(fold::fold_local(self, s))
                }
            }
            _ => fold::fold_stmt(self, s),
        }
    }
}
