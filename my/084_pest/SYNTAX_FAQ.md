<!-- markdownlint-disable MD025 -->

# Оглавление

- [Оглавление](#оглавление)
- [Подсказки синтаксиса](#подсказки-синтаксиса)
- [Общие правила](#общие-правила)
- [Количество повторений](#количество-повторений)
- [Предикаты ! и &](#предикаты--и-)
- [Приоритет](#приоритет)
- [SOI / EOI](#soi--eoi)
- [Silent и неявные пробелы с комментариями](#silent-и-неявные-пробелы-с-комментариями)
- [Атомарные правила](#атомарные-правила)
- [PUSH/POP](#pushpop)
- [Встроенные ключевые слова](#встроенные-ключевые-слова)

# Подсказки синтаксиса

Таблица синтаксиса: [ссылка](https://pest.rs/book/grammars/syntax.html#cheat-sheet)

# Общие правила

Строка может быть независимая от регистра c помощью символа ^:
`^"text"`

Диапазон значений описывается следующим образом в одинарных кавычках так как это символ: `'0'..'9'`

Существует ключевое слово ANY, которое обозначает вообще любой символ

Правила могут идти в любом порядке, а так же быть рекурсивными:

```pest
my_rule = { "slithy " ~ other_rule }
other_rule = { "toves" }
recursive_rule = { "mimsy " ~ recursive_rule }
```

Шаблоны в выражении могут быть как обернуты в скобки, так и нет - это равнозначно:

```pest
("abc") ~ (^"def" ~ ('g'..'z'))
"abc" ~ (^"def" ~ 'g'..'z')
```

# Количество повторений

Как и в регулярных выражениях, можно использовать указание конкретного количества элементов

```pest
expr{n}    // Именно n раз
expr{m, n} // Между m и n раз включительно
expr{, n}  // Не более n повторений
expr{m, }  // Как минимум m повторений
```

# Предикаты ! и &

Мы можем использовать символ & перед выражением, он значит, что предикат должен быть выполнен, тогда следующая часть будет выполнена с того же самого места.
Такую запись можно интерпретировать &foo ~ bar как текст должен соответствовать и foo, и bar правилу, но определится именно bar правило.

Можно использовать символ !, который значит, что можно обрабатывать только если правило не прошло
Обработка будет продолжаться на том же месте, но при этом правило не будет как-то обработано.
Иными словами - это просто выражение, которое не участвует в анализе, но является условием для правила.

Если указанный текст не пробел и не табуляция, тогда читаем символ дальше

```pest
not_space_or_tab = {
    !(" " | "\t") ~ ANY
}
```

Если у нас не тройные кавычки, тогда просто принимаем символ дальше

```pest
triple_quoted_character = { !"'''" ~ ANY }
```

Принимаем любые символы до тех пор пока не прилетият символы с тройными кавычками

```pest
triple_quoted_string = { "'''" ~ triple_quoted_character* ~ "'''"
```

# Приоритет

Операторы повтора имеют более высокий приоритет над операторами предиката

```pest
my_rule = { "a"* ~ "b"? | &"b"+ ~ "a" }
```

Таким образом это правило можно перефразировать вот так

```pest
my_rule = { ( ("a"*) ~ ("b"?) ) | ( (&("b"+)) ~ "a" ) }
```

# SOI / EOI

Операторы SOI + EOI лишь показывают, что мы находимся в начале и в конце входной строки
В первую очередь важен EOI, так как он показывает конец строки и что больше не будет ничего нового

# Silent и неявные пробелы с комментариями

Правила, которые не должны попадать в парсинг, должны начинаться с "_".
Если мы в нашей грамматике определяем `WHITESPACE` и/или `COMMENT`, тогда они автоматически будут подставляться в каждое правило между продолжениями. Кроме атомарных правил. 

```pest
WHITESPACE = _{ " " }
COMMENT = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
expression = {
    "4" ~ "+" ~ "5"
}
```

Развернется в:

```pest
expression = {
    "4" ~ (WHITESPACE | COMMENT)*
    ~ "+" ~ (WHITESPACE | COMMENT)*
    ~ "5"
}
```

Пример:

```pest
"4+5"
"4 + 5"
"4  +     5"
"4 /* comment */ + 5"
```

# Атомарные правила

Атомарные правила могу записываться вот так:

```pest
atomic = @{ ... }            // Обычное атомарное правило
compound_atomic = ${ ... }   // Составное атомарное правило
```

Атомарные правила вроде как занимаются тем, что не позволяют неявным пробелам оказаться в тексте?
В обычных атомиках внутренние правила silent, то есть не показываются в выводе.
В составных внутренние правила показываются как и обычные при парсинге, но пробелов или комментриев через WHITESPACE/COMMENT там быть не может.

# PUSH/POP

Грамматика имеет возможность сохранять выражения в стеке.
Для этого служат ключевые слова PUSH/POP/PEEK.

Сначала мы пушим выражение, затем либо извлекаем, либо вызываем PEEK просто для указания

```pest
 same_text = {
    PUSH( "a" | "b" | "c" )
    ~ PEEK
    ~ POP
}
same_pattern = {
    ("a" | "b" | "c") ~ 
    ("a" | "b" | "c") ~ 
    ("a" | "b" | "c")
}
```

Это может быть использовано для того, чтобы обрабатывать открывающиеся и закрывающиеся r###/###, когда мы не знаем точного количества элементов, которые встретятся.

Сам стек является глобальным, поэтому может быть использован и в другом правиле.

```pest
raw_string = {
     "r" ~ PUSH("#"*) ~ "\""    // Сохраняем количество найденных символов в стек
     ~ raw_string_interior
     ~ "\"" ~ POP               // затем должно быть аналогичное количество элементов из стека
}
```

Затем пока у нас не встречается закрывающая кавычка и нужное количество элементов на стеке - принимаем любые символы

```pest
raw_string_interior = {
     ( !("\"" ~ PEEK))* ~ ANY )
}
```

# Встроенные ключевые слова

[Ссылка](https://pest.rs/book/grammars/built-ins.html) на правила.
