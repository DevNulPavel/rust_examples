// Так мы подключаем что-то из соседнего файлика
use crate::scanner::Token;

// Указываем, чтобы компилятор автоматически реализовал трейты, полезно для типа ошибок
#[derive(Debug, PartialEq)]
pub enum Error {
    // Если нам нужно унаследовать что-то, то пишем в скобках тип
    UnexpectedChar(u8),
    UnexpectedToken(Token),
}
