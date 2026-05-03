use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Comparator {
    Less,
    LessOrEq,
    Greater,
    GreaterOrEq,
    Eq,
    NotEq,
}

// FieldPath - это путь до поля, например user.geo.country -> vec!["user", "geo", "country"]
pub type FieldPath = Vec<String>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Empty,
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),

    Flag(FieldPath),
    Exists(FieldPath),
    NotExists(FieldPath),

    // Универсальное сравнение любых JSON типов (user.score >= 10, geo.country == "US")
    Cmp(FieldPath, Comparator, JsonValue),

    // Массивы (PRIMARY_LANGUAGE AMONG["en", "ru"])
    In(FieldPath, Vec<JsonValue>),

    // Время (Будильники)
    AgeComp(FieldPath, Comparator, u64),
    TimeCompNow(FieldPath, Comparator),

    // Модуль (user_id MOD 10 == 5)
    ModComp(FieldPath, Comparator, u64, u64),

    // Побитовое И
    BitComp(FieldPath, Comparator, u64, u64),

    Contains(FieldPath, String),
    Icontains(FieldPath, String),

    Any(FieldPath, Box<Expr>),
    All(FieldPath, Box<Expr>),

    // Проверка длины
    LenComp(FieldPath, Comparator, u64),
}
