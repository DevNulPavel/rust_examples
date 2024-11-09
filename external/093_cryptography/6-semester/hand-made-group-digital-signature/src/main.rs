use hand_made_group_digital_signature::{
    check_sign, IngroupKeys, KeyComponents, PersonalKeys
};
use hand_made_sha::sha256;
use num::{BigInt, One};

fn main() {
    let message = "hello world!";

    // Подготовка
    let kc = KeyComponents::new();
    // dbg!(&kc);

    let leader_pk = PersonalKeys::new(&kc);
    let user_1_pk = PersonalKeys::new(&kc);
    let user_2_pk = PersonalKeys::new(&kc);
    // dbg!(&leader_pk, &user_1_pk, &user_2_pk);

    let leader_ik = IngroupKeys::new();
    // dbg!(&leader_ik);

    // Шаг 1
    let hash = &BigInt::from_signed_bytes_be(&sha256(message.as_bytes()));

    let users_personal_keys = vec![&user_1_pk, &user_2_pk];
    let mut users_lambda: Vec<BigInt> = vec![];
    for i in &users_personal_keys {
        users_lambda.push((hash + i.get_public_key()).modpow(leader_ik.get_d(), leader_ik.get_n()))
    }
    // dbg!(&users_lambda);

    let mut U = BigInt::one();
    for i in 0..users_lambda.len() {
        U *= users_personal_keys[i].get_public_key().modpow(&users_lambda[i], kc.get_p());
    }
    U %= kc.get_p();
    // dbg!(&U);

    // Шаг 2-3
    let leader_sk = PersonalKeys::new(&kc);
    let user_1_sk = PersonalKeys::new(&kc);
    let user_2_sk = PersonalKeys::new(&kc);
    // dbg!(&leader_sk, &user_1_sk, &user_2_sk);

    let users_session_keys = vec![&user_1_sk, &user_2_sk];
    let mut R = BigInt::one();
    for i in 0..users_session_keys.len() {
        R *= users_session_keys[i].get_public_key() % kc.get_p();
    }
    R *= leader_sk.get_public_key() % kc.get_p();
    R %= kc.get_p();
    // dbg!(&R);

    let mut temp = Vec::from(message.as_bytes());
    temp.extend_from_slice(&R.to_signed_bytes_be());
    temp.extend_from_slice(&U.to_signed_bytes_be());
    let E = BigInt::from_bytes_be(num_bigint::Sign::Plus, &sha256(&temp));
    // dbg!(&E);

    // Шаг 4
    let mut users_signs: Vec<BigInt> = vec![];
    for i in 0..users_lambda.len() {
        let t = users_session_keys[i].get_private_key();
        let k = users_personal_keys[i].get_private_key();
        let lambda = &users_lambda[i];
        let q = kc.get_q();

        users_signs.push((t + k * lambda * &E) % q);
    }
    // dbg!(&users_signs);

    // Шаг 5
    let mut temp_users_public_keys: Vec<BigInt> = vec![];
    for i in 0..users_lambda.len() {
        let public_key = users_personal_keys[i].get_public_key();
        let lambda = &users_lambda[i];
        let sign = &users_signs[i];
        let alpha = kc.get_alpha();
        let p = kc.get_p();

        temp_users_public_keys.push((public_key.modinv(p).unwrap().modpow(&(lambda * &E), p) * alpha.modpow(sign, p)) % p);
    }
    // dbg!(&temp_users_public_keys);

    let leader_sign = (leader_sk.get_private_key() + leader_pk.get_private_key() * &E) % kc.get_q();
    let mut S = leader_sign;
    for i in users_signs {
        S += i;
    }
    S %= kc.get_q();
    // dbg!(&S);

    println!("Групповая подпись\nU: {:?}\nE: {:?}\nS: {:?}\nMessage: {}", &U, &E, &S, message);

    println!("Проверка ...");

    let L = leader_pk.get_public_key();
    let p = kc.get_p();
    let alpha = kc.get_alpha();
    let probably_R = (&U * L).modpow(L, p).modinv(p).unwrap() * alpha.modpow(&S, p);
    // dbg!(&probably_R);

    let mut temp = Vec::from(message.as_bytes());
    temp.extend_from_slice(&probably_R.to_signed_bytes_be());
    temp.extend_from_slice(&U.to_signed_bytes_be());
    let probably_E = BigInt::from_bytes_be(num_bigint::Sign::Plus, &sha256(&temp));
    // dbg!(&probably_E);

    if check_sign(&E, &probably_E) {
        println!("Групповая подпись верна\nU: {:?}\nE: {:?}\nS: {:?}\nMessage: {}", &U, &E, &S, message);
    } else {
        println!("Групповая подпись неверна\nU: {:?}\nE: {:?}\nS: {:?}\nMessage: {}", &U, &E, &S, message);
    }
}

// / Генерация ключевых компонентов
// / $$p - \text{простое}$$
// / $$q - \text{простое} и q \equiv 0 \pmod{p - 1}$$
// / $$\alpha: ord(\alpha) = q$$
// /
// / Генерация ключей
// / $$public_key = \alpha^{private_key} \pmod{p}$$
// /
// / Генерация внутригруппового ключа лидера
// / $$p_1 \text{и} p_2 - \text{простые}$$
// / $$n = p_1 \cdot p_2$$
// / $$e \cdot d \equiv 1 \pmod{\varphi(n)}$$
// / $$\text{Открытый ключ: } e, n$$
// / $$\text{Закрытый ключ: } p_1, p_2, d$$
// /
// / Вычисление родписи
// / H = h(M)
// fn main() {
//     println!("Генерация компонентов ключей");

//     let (p, q, a) = generate_key_components();

//     println!("Готово");

//     println!("Пользователи генерируют ключи");

//     // Пользователи генерируют себе ключи
//     let users = [
//         generate_keys_from_components(&a, &p),
//         generate_keys_from_components(&a, &p),
//         generate_keys_from_components(&a, &p),
//     ];

//     println!("Готово");

//     println!("Лидер генерирует ключ");

//     // Лидер генерирует себе ключи
//     let (z, L) = generate_keys_from_components(&a, &p);

//     println!("Готово");

//     println!("Лидер генерирует ключ");

//     // Лидер генерирует себе внутригрупповые ключи
//     let (e, n, d) = generate_ingroup_leader_key();

//     println!("Готово");

//     // Сообщение
//     let message = BigInt::from_signed_bytes_be("Hello world".as_bytes());

//     println!("Лидер вычисляет значения лямбда");

//     // Лидер вычисляет для пользователей значения лямбда
//     let message_hash = BigInt::from_signed_bytes_be(&sha256(&message.to_signed_bytes_be()));
//     let mut lambda: Vec<BigInt> = vec![];
//     for user_keys in &users {
//         lambda.push(calc_lambda(&message_hash, &user_keys.1, &d, &n));
//     }

//     println!("Готово");

//     let U = calc_u(&users.clone().map(|keys| keys.1), &lambda, &p);

//     // Пользователи генерируют сессионные ключи
//     let rt = [
//         generate_session_keys_from_components(&a, &p, &q),
//         generate_session_keys_from_components(&a, &p, &q),
//         generate_session_keys_from_components(&a, &p, &q),
//     ];

//     let r = [rt[0].1.clone(), rt[1].1.clone(), rt[2].1.clone()];

//     // Лидер расчитывает значение E
//     let (T, E) = calc_e(&a, &p, &q, &r, &message, &U);

//     // Пользователи вычисляют компоненты групповой подписи S_i
//     let mut S_i: Vec<BigInt> = vec![];
//     for i in 0..rt.len() {
//         S_i.push(calc_s_i(&rt[i].0, &users[i].0, &lambda[i], &E, &q));
//     }

//     // Лидер проверяет сессионные ключи пользователей
//     let mut flag = true;
//     for i in 0..rt.len() {
//         flag = check_session_key(&rt[i].1, &users[i].1, &lambda[i], &E, &a, &S_i[i], &p);
//         if !flag {
//             break;
//         }
//     }
//     if !flag {
//         panic!("сессионные ключи не действительны")
//     }

//     let S = calc_s(&T, &z, &E, &q, &S_i);

//     let sign = Sign::new(&U, &E, &S);

//     if sign.check(&message, &U, &L, &a, &S, &p, &E) {
//         println!("Подпись верна");
//     } else {
//         println!("Подпись неверна");
//     }
// }
