use logos::Logos;

pub mod ast;
pub mod evaluator;
pub mod lexer;

// Подключаем сгенерированный парсер
lalrpop_util::lalrpop_mod!(pub parser);

/// Главная функция парсинга строки в AST
pub fn parse_query(input: &str) -> Result<Box<ast::Expr>, String> {
    if input.trim().is_empty() {
        return Ok(Box::new(ast::Expr::Empty));
    }

    let lexer = lexer::Token::lexer(input);

    // Преобразуем токены Logos в формат, ожидаемый LALRPOP
    let stream = lexer.spanned().map(|(token_res, span)| {
        match token_res {
            Ok(token) => Ok((span.start, token, span.end)),
            Err(_) => Err(span.start), // Передаем позицию ошибки
        }
    });

    let parser = parser::QueryParser::new();
    parser
        .parse(stream)
        .map_err(|e| format!("Parse error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use crate::evaluator::Evaluator;

    use super::*;
    use ast::{Comparator, Expr};
    use serde_json::json;

    #[test]
    fn test_among_array_selector() {
        // Запрос чистый и без хардкода!
        let query = "\
            enabled AND \
            (\
                (last_check AGE a > 5 minutes) OR \
                (last_check NOT EXISTS) \
            )\
            AND important \
        ";

        let ast = parse_query(query).unwrap();

        // Все ключи теперь - просто динамические массивы строк
        assert_eq!(
            *ast,
            Expr::And(
                Box::new(Expr::And(
                    Box::new(Expr::Flag(vec!["enabled".to_string()])),
                    Box::new(Expr::Or(
                        Box::new(Expr::AgeComp(
                            vec!["last_check".to_string()],
                            Comparator::Greater,
                            300
                        )),
                        Box::new(Expr::NotExists(vec!["last_check".to_string()]))
                    ))
                )),
                Box::new(Expr::Flag(vec!["important".to_string()]))
            )
        );
    }

    #[test]
    fn test_foreign_users_selector() {
        // Тот самый тест, который падал!
        // Теперь NOT AMONG работает для любых путей.
        let query = "\
            enabled AND \
            (last_check AGE a > 5 minutes) AND \
            (primary_language NOT AMONG[\"en\", \"ru\"]) AND \
            NOT important\
        ";

        let ast = parse_query(query).unwrap();

        let meta = json!({
            "enabled": true,
            "last_check": 1000,
            "primary_language": "de", // Не en и не ru
            "important": false        // NOT important
        });

        let result = Evaluator::evaluate(&ast, &meta, 1500);
        assert_eq!(result.is_match, true);
    }

    #[test]
    fn test_new_now_syntax() {
        let query = "last_check < NOW() AND next_check_time >= NOW()";
        let ast = parse_query(query).unwrap();

        assert_eq!(
            *ast,
            Expr::And(
                Box::new(Expr::TimeCompNow(
                    vec!["last_check".to_string()],
                    Comparator::Less
                )),
                Box::new(Expr::TimeCompNow(
                    vec!["next_check_time".to_string()],
                    Comparator::GreaterOrEq
                ))
            )
        );
    }

    #[test]
    fn test_mod_parsing_and_evaluation() {
        let query = "user_id MOD 10 == 5";
        let ast = parse_query(query).unwrap();

        assert_eq!(
            *ast,
            Expr::ModComp(vec!["user_id".to_string()], Comparator::Eq, 10, 5)
        );

        let meta = json!({ "user_id": 12345 });
        let result = Evaluator::evaluate(&ast, &meta, 0);
        assert_eq!(result.is_match, true);

        let meta_no_match = json!({ "user_id": 12340 });
        let result_no_match = Evaluator::evaluate(&ast, &meta_no_match, 0);
        assert_eq!(result_no_match.is_match, false);
    }

    bitflags::bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct Flags: u8 {
            const ENABLED   = 0b00000001; // 1
            const IMPORTANT = 0b00000010; // 2
            const PREMIUM   = 0b00000100; // 4
        }
    }

    #[test]
    fn test_bitwise_flags() {
        let query = format!(
            "status_flags & {} == {}",
            Flags::IMPORTANT.bits(),
            Flags::IMPORTANT.bits()
        );
        let ast = parse_query(&query).unwrap();

        assert_eq!(
            *ast,
            Expr::BitComp(vec!["status_flags".to_string()], Comparator::Eq, 2, 2)
        );

        let user1_flags = Flags::ENABLED | Flags::IMPORTANT;
        let meta1 = json!({ "status_flags": user1_flags.bits() });
        let res1 = Evaluator::evaluate(&ast, &meta1, 0);
        assert_eq!(res1.is_match, true);

        let user2_flags = Flags::ENABLED | Flags::PREMIUM;
        let meta2 = json!({ "status_flags": user2_flags.bits() });
        let res2 = Evaluator::evaluate(&ast, &meta2, 0);
        assert_eq!(res2.is_match, false);

        let query_combo = format!(
            "status_flags & {} == {}",
            (Flags::ENABLED | Flags::PREMIUM).bits(),
            (Flags::ENABLED | Flags::PREMIUM).bits()
        );
        let ast_combo = parse_query(&query_combo).unwrap();
        let res3 = Evaluator::evaluate(&ast_combo, &meta2, 0);
        assert_eq!(res3.is_match, true);
    }

    #[test]
    fn test_string_contains() {
        let query_cs = "username CONTAINS \"Admin\"";
        let ast_cs = parse_query(query_cs).unwrap();

        let query_ic = "username ICONTAINS \"admin\"";
        let ast_ic = parse_query(query_ic).unwrap();

        let meta_exact = json!({ "username": "SuperAdmin123" });
        assert_eq!(Evaluator::evaluate(&ast_cs, &meta_exact, 0).is_match, true);
        assert_eq!(Evaluator::evaluate(&ast_ic, &meta_exact, 0).is_match, true);

        let meta_lower = json!({ "username": "superadmin123" });
        assert_eq!(Evaluator::evaluate(&ast_cs, &meta_lower, 0).is_match, false);
        assert_eq!(Evaluator::evaluate(&ast_ic, &meta_lower, 0).is_match, true);

        let meta_none = json!({ "username": "SuperUser" });
        assert_eq!(Evaluator::evaluate(&ast_cs, &meta_none, 0).is_match, false);
        assert_eq!(Evaluator::evaluate(&ast_ic, &meta_none, 0).is_match, false);
    }

    #[test]
    fn test_array_any_complex_element() {
        let query =
            "devices ANY (device.type AMONG [\"ios\", \"android\"] AND added_at AGE < 14 hours)";
        let ast = parse_query(query).unwrap();

        let now_ts = 100_000;

        let meta_match = json!({
            "devices": [
                { "device": {"type": "windows"}, "added_at": now_ts - 10 }, // Не подходит по AMONG
                { "device": {"type": "ios"}, "added_at": now_ts - 3600 }    // Подходит (1 час < 14 часов)
            ]
        });

        let res1 = Evaluator::evaluate(&ast, &meta_match, now_ts);
        assert_eq!(res1.is_match, true);

        let meta_too_old = json!({
            "devices": [
                { "device": {"type": "android"}, "added_at": now_ts - (15 * 3600) }
            ]
        });
        let res2 = Evaluator::evaluate(&ast, &meta_too_old, now_ts);
        assert_eq!(res2.is_match, false);

        let meta_wrong_type = json!({
            "devices": [
                { "device": {"type": "linux"}, "added_at": now_ts - 100 }
            ]
        });
        let res3 = Evaluator::evaluate(&ast, &meta_wrong_type, now_ts);
        assert_eq!(res3.is_match, false);
    }

    #[test]
    fn test_all_operator() {
        let query = "devices ALL (status == \"ok\" AND errors_count == 0)";
        let ast = parse_query(query).unwrap();

        let meta_match = json!({
            "devices": [
                { "status": "ok", "errors_count": 0 },
                { "status": "ok", "errors_count": 0 }
            ]
        });
        assert_eq!(Evaluator::evaluate(&ast, &meta_match, 0).is_match, true);

        let meta_fail = json!({
            "devices": [
                { "status": "ok", "errors_count": 0 },
                { "status": "error", "errors_count": 5 }
            ]
        });
        assert_eq!(Evaluator::evaluate(&ast, &meta_fail, 0).is_match, false);

        let meta_empty = json!({ "devices": [] });
        assert_eq!(Evaluator::evaluate(&ast, &meta_empty, 0).is_match, true);

        let meta_null = json!({ "other_field": true });
        assert_eq!(Evaluator::evaluate(&ast, &meta_null, 0).is_match, false);
    }

    #[test]
    fn test_len_operator() {
        let query = "tags LEN >= 2 AND profile LEN == 3 AND username LEN < 6";
        let ast = parse_query(query).unwrap();

        let meta = json!({
            "tags": ["vip", "active"],           // массив (длина 2)
            "profile": {                         // объект (длина 3 ключа)
                "age": 25,
                "country": "US",
                "gender": "M"
            },
            "username": "admin"                  // строка (длина 5)
        });

        assert_eq!(Evaluator::evaluate(&ast, &meta, 0).is_match, true);
    }

    #[test]
    fn test_array_of_nulls_with_this() {
        let query = "items ALL (SELF == null OR SELF == \"ok\")";
        let ast = parse_query(query).unwrap();

        let meta_all_null = json!({
            "items": [null, null, null]
        });
        assert_eq!(Evaluator::evaluate(&ast, &meta_all_null, 0).is_match, true);

        let meta_err = json!({
            "items": [null, "error", null]
        });
        assert_eq!(Evaluator::evaluate(&ast, &meta_err, 0).is_match, false);

        let meta_ok = json!({
            "items": [null, "ok", "ok"]
        });
        assert_eq!(Evaluator::evaluate(&ast, &meta_ok, 0).is_match, true);
    }

    #[test]
    fn test_evaluator_expiration() {
        // AGE expiration
        let query_age = "created_at AGE < 300 seconds";
        let ast_age = parse_query(query_age).unwrap();
        let meta_age = json!({ "created_at": 1000 });

        let res_age = Evaluator::evaluate(&ast_age, &meta_age, 1100);
        assert_eq!(res_age.is_match, true);
        assert_eq!(res_age.wake_up_at, Some(1300)); // Должен проснуться чтобы выкинуть из очереди

        let query_now = "expire_ts > NOW()";
        let ast_now = parse_query(query_now).unwrap();
        let meta_now = json!({ "expire_ts": 2000 });

        // Ровно в 2000 секунд перестанет подходить.
        let res_now = Evaluator::evaluate(&ast_now, &meta_now, 1500);
        assert_eq!(res_now.is_match, true);
        assert_eq!(res_now.wake_up_at, Some(2000));
    }

    #[test]
    fn test_empty_selector() {
        // Парсим пустую строку или строку с пробелами
        let query = "  ";
        let ast = parse_query(query).unwrap();

        assert_eq!(*ast, Expr::Empty);

        // Создаем абсолютно любого юзера (даже с пустой метой)
        let meta = json!({ "some_random_field": 42 });

        // Проверяем
        let result = Evaluator::evaluate(&ast, &meta, 0);

        // Он должен идеально подойти!
        assert_eq!(result.is_match, true);
        assert_eq!(result.wake_up_at, None);
    }

    #[test]
    fn test_evaluator_age_comp_future_wakeup() {
        let query = "last_check AGE > 5 minutes";
        let ast = parse_query(query).unwrap();

        // Смотрите как красиво описывается юзер! Любые вложенности и JSON типы.
        let meta = json!({
            "last_check": 1000,
            "some_array": [1, 2, 3],
            "nested_obj": { "key": "value" }
        });

        let now_ts = 1100;
        let result = Evaluator::evaluate(&ast, &meta, now_ts);

        assert_eq!(result.is_match, false);
        assert_eq!(result.wake_up_at, Some(1301));
    }

    #[test]
    fn test_complex_and_logic_wakeup() {
        let query = "enabled AND (last_check AGE > 5 minutes) AND (next_check <= NOW())";
        let ast = parse_query(query).unwrap();

        let meta = json!({
            "enabled": true,
            "last_check": 1000,
            "next_check": 2000
        });

        let now_ts = 1200;
        let result = Evaluator::evaluate(&ast, &meta, now_ts);

        assert_eq!(result.is_match, false);
        assert_eq!(result.wake_up_at, Some(2000));
    }

    #[test]
    fn test_evaluator_ready_now_with_json_types() {
        let query = "enabled AND primary_language AMONG [en, ru] AND last_check EXISTS";
        let ast = parse_query(query).unwrap();

        let meta = json!({
            "enabled": true,
            "primary_language": "ru",
            "last_check": 1711900000,
            "null_field": null // Полная поддержка JSON Null
        });

        let result = Evaluator::evaluate(&ast, &meta, 0);

        assert_eq!(result.is_match, true);
        assert_eq!(result.wake_up_at, None);
    }

    #[test]
    fn test_nested_json_paths_and_types() {
        // Запрос с вложенными путями, строками и числами!
        let query = "\
            user.geo.country == \"US\" AND \
            stats.score >= 100.5 AND \
            is_premium == true AND \
            user.banned == false \
        ";
        let ast = parse_query(query).unwrap();

        // Профиль юзера (Сложный JSON объект)
        let meta = json!({
            "is_premium": true,
            "user": {
                "geo": {
                    "country": "US"
                },
                "banned": false
            },
            "stats": {
                "score": 150.75
            }
        });

        let result = Evaluator::evaluate(&ast, &meta, 0);

        // Идеальное совпадение!
        assert_eq!(result.is_match, true);
    }

    #[test]
    fn test_arrays_among_with_nested_paths() {
        // Мы ищем пользователя, у которого тег устройства либо iOS, либо Android
        let query = "device.os AMONG [\"iOS\", \"Android\"] AND subscription == null";
        let ast = parse_query(query).unwrap();

        let meta = json!({
            "device": { "os": "Android" },
            "subscription": null // Проверяем JSON Null
        });

        let result = Evaluator::evaluate(&ast, &meta, 0);
        assert_eq!(result.is_match, true);
    }

    #[test]
    fn test_arrays_among_with_list_field() {
        // Мы ищем пользователя, у которого в primary_languages есть хотя бы один из [de, fr]
        let query = "primary_languages AMONG [\"de\", \"fr\"]";
        let ast = parse_query(query).unwrap();

        let meta = json!({
            "primary_languages": ["en", "de", "es"] // "de" совпадает с одним из [de, fr]
        });

        let result = Evaluator::evaluate(&ast, &meta, 0);
        assert_eq!(result.is_match, true);

        // Проверим также NOT AMONG для массивов
        let query_not = "primary_languages NOT AMONG [\"de\", \"fr\"]";
        let ast_not = parse_query(query_not).unwrap();
        let result_not = Evaluator::evaluate(&ast_not, &meta, 0);
        assert_eq!(result_not.is_match, false);
    }
}
