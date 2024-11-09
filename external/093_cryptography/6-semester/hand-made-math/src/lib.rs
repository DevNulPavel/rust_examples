#![allow(non_snake_case)]
use num_bigint::{BigInt, BigUint, RandBigInt, ToBigInt};
use num_traits::{Num, One, ToPrimitive, Zero};
use rand::{Rng, RngCore};
use std::time::{Duration, Instant};

/// Расширенный алгоритм евклида
pub fn expanded_euclidean_algorithm<T: Into<BigInt>>(a: T, b: T) -> (BigInt, BigInt, BigInt) {
    let a: BigInt = a.into();
    let b: BigInt = b.into();

    if &b == &BigInt::zero() {
        return (a.clone(), 1.to_bigint().unwrap(), 0.to_bigint().unwrap());
    }
    let (greatest_divisor, u, v) = expanded_euclidean_algorithm(b.clone(), &a % &b);
    let v1 = u - (a / b) * &v;
    let u1 = v;

    (greatest_divisor, u1, v1)
}

pub fn rand_big_int(bits: u64) -> BigInt {
    let mut rng = rand::thread_rng();
    let number = BigInt::from(rng.gen_biguint(bits));
    number
}

pub fn rand_big_int_exactly(bits: u64) -> BigInt {
    let mut rng = rand::thread_rng();
    let mut random_number = BigInt::zero();
    for _ in 0..bits {
        random_number = (random_number << 1) + BigInt::from(rng.gen_range(0..2));
    }
    random_number
}

/// Быстрое возведение в степень
pub fn exponentiation<T: Into<BigInt>>(number: T, pow: T) -> BigInt {
    let number = number.into();
    let pow = pow.into();

    let mut binary_code: Vec<u32> = pow
        .to_str_radix(2)
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    binary_code.reverse();

    let mut result = BigInt::one();
    for i in 0..binary_code.len() {
        result = &result * number.pow(binary_code[i] * 2_u32.pow(i.try_into().unwrap()));
    }
    result
}

/// Быстрое возведение в степень по модулю
pub fn module_exponentiation<T: Into<BigInt>>(number: T, pow: T, module: T) -> BigInt {
    let number = number.into();
    let pow = pow.into();
    let module = module.into();

    let mut binary_code: Vec<u32> = pow
        .to_str_radix(2)
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    binary_code.reverse();

    let mut y = number.clone();
    let mut x = number.pow(binary_code[0]);

    for i in 1..binary_code.len() {
        y = y.pow(2) % module.clone();
        if binary_code[i] == 1_u32 {
            x = (x * y.clone()) % module.clone();
        }
    }

    x
}

/// Вычисление символа Якоби
pub fn calculate_symbol_jacobi<T: Into<BigInt>>(a: T, n: T) -> Result<BigInt, &'static str> {
    let mut a = a.into();
    let mut n = n.into();

    if &n < &BigInt::from(3) {
        return Err("Число n < 3");
    }
    // if &a >= &n || &a < &BigInt::zero() {
    //     return Err("Число а не в промежутке от 0 до n");
    // }

    let mut g = 1;

    loop {
        if &a == &BigInt::zero() {
            return Ok(BigInt::zero());
        }
        if &a == &BigInt::one() {
            return Ok(BigInt::from(g));
        }

        let k = count_twos(a.clone());
        let a1 = &a / BigInt::from(2).pow(k);

        let mut s = define_s(BigInt::from(k), n.clone());

        if a1 == BigInt::one() {
            return Ok(BigInt::from(g * s));
        }

        if &n % 4 == BigInt::from(3) && &a1 % 4 == BigInt::from(3) {
            s *= -1;
        }

        a = &n % &a;
        n = a1;
        g *= s;
    }
}

fn count_twos(mut num: BigInt) -> u32 {
    let mut count: u32 = 0;

    loop {
        if &num % 2 == BigInt::zero() {
            count += 1;
            num = num / 2;
        } else {
            break;
        }
    }
    count
}

fn define_s(k: BigInt, n: BigInt) -> i32 {
    if k % 2 == BigInt::zero() {
        return 1;
    }
    if &n % 8 == BigInt::one() || &n % 8 == BigInt::from(7) {
        return 1;
    } else if &n % 8 == BigInt::from(3) || &n % 8 == BigInt::from(5) {
        return -1;
    }
    1
}

/// Теста ферма на простоту числа
pub fn test_ferma<T: Into<BigInt>>(n: T) -> &'static str {
    let n = n.into();

    if &n < &BigInt::from(5) {
        return "n меньше 5, неверно";
    }
    let mut rng = rand::thread_rng();
    let a = rng.gen_bigint_range(&BigInt::from(2), &(n.clone() - BigInt::from(2)));
    let r = module_exponentiation(a, n.clone() - 1, n.clone());

    if r == BigInt::one() {
        return "Число n, вероятно простое";
    }
    "Число n составное"
}

pub fn check_time_function<F, T>(f: F) -> (Duration, T)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    (start.elapsed(), result)
}

/// Решение сравнения первой степени
pub fn solve_comparison<T: Into<BigInt>>(a: T, b: T, module: T) -> Option<Vec<BigInt>> {
    let a = a.into();
    let b = b.into();
    let module = module.into();

    let (gcd, _u, v) = expanded_euclidean_algorithm(a.clone(), module.clone());
    let mut results: Option<Vec<BigInt>> = Some(vec![]);

    if &b % &gcd != BigInt::zero() {
        results = None;
        return results;
    }

    if &gcd != &BigInt::one() {
        let a1 = a / &gcd;
        let b1 = b / &gcd;
        let module1 = &module / &gcd;
        let (_gcd1, u1, _v1) = expanded_euclidean_algorithm(a1.clone(), module1.clone());

        let result = u1 * b1 % &module1;
        results.as_mut().unwrap().push(result.clone());

        for i in 1..gcd.to_i128().unwrap() {
            results
                .as_mut()
                .unwrap()
                .push(&result + &module1 * BigInt::from(i));
        }
        results
    } else {
        results.as_mut().unwrap().push(b * v % &module);
        results
    }
}

/// Тест Соловея-Штрассена на простоту
pub fn test_solovei_shtrassen<T: Into<BigInt>>(n: T) -> &'static str {
    let n = n.into();

    if &n % 2 == BigInt::zero() || &n < &BigInt::from(5) {
        return "Число n либо чётное, либо меньше 5, ошибка";
    }

    let mut rng = rand::thread_rng();
    let a = rng.gen_bigint_range(&BigInt::from(2), &(n.clone() - BigInt::from(2)));

    let r = module_exponentiation(a.clone(), BigInt::from((&n - 1) / 2), n.clone());

    if &r != &BigInt::one() && r != &n - 1 {
        return "Число n составное";
    }

    let mut s = calculate_symbol_jacobi(a.clone(), n.clone()).unwrap();

    if &s == &BigInt::from(-1) {
        s += &n;
    }

    if &r % &n == s {
        return "Число n, вероятно, простое";
    }
    return "Число n составное";
}

/// Тест Миллера-Рабина на простоту
pub fn test_miller<T: Into<BigInt>>(n: BigInt) -> &'static str {
    let n: BigInt = n.into();

    if &n % 2 == BigInt::zero() || &n < &BigInt::from(5) {
        return "Число n либо чётное, либо меньше 5, ошибка";
    }

    let s = count_twos(&n - 1);

    let mut j: u32;

    let mut rng = rand::thread_rng();
    let a = rng.gen_bigint_range(&BigInt::from(2), &(n.clone() - BigInt::from(2)));

    let r = (&n - 1) / 2_i32.pow(s);

    let mut y = module_exponentiation(a.clone(), r, n.clone());

    if &y != &BigInt::from(1) && y != &n - 1 {
        j = 1;

        while j <= s - 1 && y != &n - 1 {
            y = module_exponentiation(y.clone(), BigInt::from(2), n.clone());
            if &y == &BigInt::one() {
                return "Число n составное";
            }
            j += 1
        }
        if y != &n - 1 {
            return "Число n составное";
        }
    }

    "Число n, вероятно, простое"
}

/// Генерация простого числа заданной размерности k и точности t
pub fn prime_number_generation(k: u64, t: u64) -> BigInt {
    let mut rng = rand::thread_rng();
    let mut bits = String::new();
    bits.push('1');

    for _el in 0..k - 2 {
        let temp = rng.gen_range(0..2);
        if temp == 0 {
            bits.push('0');
        } else if temp == 1 {
            bits.push('1');
        }
    }
    bits.push('1');

    let mut k_number = BigInt::from_str_radix(bits.as_str(), 2).unwrap();

    for mut _i in 1..=t {
        let a = rng.gen_bigint_range(&BigInt::from(2), &BigInt::from(&k_number - 2));
        match test_miller_rabin_for_prime_number::<BigInt>(k_number.clone(), a) {
            "Число n, вероятно, простое" => {}
            _ => k_number = prime_number_generation(k, t),
        }
    }
    k_number
}

/// # Генерация простого числа
/// - `size` - размер числа в байтах
/// - `iters` - количество проверок числа на простоту
pub fn rpn(size: usize, iters: usize) -> BigInt {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; size];
    let mut number: BigInt;

    'main: loop {
        rng.fill_bytes(&mut bytes);
        bytes[0] = bytes[0] | 0b10000000;
        bytes[size - 1] = bytes[size - 1] | 0b00000001;

        number = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes);

        for _ in 0..iters {
            match miller_rabin_test(&number.clone().try_into().unwrap()) {
                true => break 'main,
                _ => continue,
            }
        }
    }

    number
}

/// # Тест Миллера-Рабина на простоту числа
/// - true - число простое
/// - false - число составное
pub fn miller_rabin_test(number: &BigUint) -> bool {
    if number < &BigUint::from(5u8) {
        return false;
    } else if number % BigUint::from(2u8) == BigUint::from(2u8) {
        return false;
    }

    let mut s = BigUint::zero();
    let mut r = number - BigUint::one();
    while r.clone() % BigUint::from(2u8) == BigUint::zero() {
        s += BigUint::one();
        r = r >> 1;
    }

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u8), &(number - BigUint::from(2u8)));
    let mut y = a.modpow(&r, &number);

    while y != BigUint::one() && y != (number - BigUint::one()) {
        let mut j = BigUint::one();

        while j <= (s.clone() - BigUint::one()) && y != (number - BigUint::one()) {
            y = y.modpow(&BigUint::from(2u8), &number);
            if y == BigUint::one() {
                return false;
            }
            j += BigUint::one();
        }

        if y != (number - BigUint::one()) {
            return false;
        }
    }

    true
}

/// Тест Миллера-Рабина на простоту числа
pub fn test_miller_rabin_for_prime_number<T: Into<BigInt>>(n: BigInt, a: BigInt) -> &'static str {
    let n: BigInt = n.into();
    let a: BigInt = a.into();

    if &n % 2 == BigInt::zero() {
        return "Число n чётное";
    } else if &n < &BigInt::from(5) {
        return "Число n меньше 5";
    }

    let mut j: u32;
    let s = count_twos(&n - 1);
    let r = (&n - 1) / 2_i32.pow(s);
    let mut y = module_exponentiation(a.clone(), r, n.clone());

    if &y != &BigInt::from(1) && y != &n - 1 {
        j = 1;

        while j <= s - 1 && y != &n - 1 {
            y = module_exponentiation(y.clone(), BigInt::from(2), n.clone());
            if &y == &BigInt::one() {
                return "Число n составное";
            }
            j += 1
        }
        if y != &n - 1 {
            return "Число n составное";
        }
    }

    "Число n, вероятно, простое"
}

/// Решения сравнения второй степени
pub fn solve_comparison_second_degree<T: Into<BigInt>>(
    p: BigInt,
    a: BigInt,
    N: BigInt,
) -> Result<BigInt, &'static str> {
    let p: BigInt = p.into();
    let a: BigInt = a.into();
    let N: BigInt = N.into();

    if p == BigInt::from(2) {
        return Err("Число p равно двум");
    }
    if calculate_symbol_jacobi(a.clone(), p.clone()).unwrap() != BigInt::from(1)
        || BigInt::from(-1) * calculate_symbol_jacobi(N.clone(), p.clone()).unwrap()
            != BigInt::from(1)
    {
        return Err("Числа а, N не подходят");
    }

    let k = count_twos(&p - 1);
    let h: BigInt = (&p - 1) / BigInt::from(2).pow(k);

    let a1 = module_exponentiation(a.clone(), (&h + 1) / 2, p.clone());
    let (_, u, _v) = expanded_euclidean_algorithm(a.clone(), p.clone());
    let a2 = u.clone(); //&&&

    let N1 = module_exponentiation(N.clone(), h.clone(), p.clone());
    let mut N2 = BigInt::from(1);
    let mut j = Vec::new();

    let k = k as i32;

    for i in 0..=(k - 2) {
        let b = (&a1 * &N2) % &p;
        let c = (&a2 * &b.pow(2)) % &p;
        let d = module_exponentiation(
            c.clone(),
            BigInt::from(2).pow((k - 2 - i).try_into().unwrap()),
            p.clone(),
        );

        if d == BigInt::from(1) {
            j.push(0);
        } else if d == BigInt::from(-1) || d == &p - 1 {
            j.push(1);
        }
        N2 = N2
            * module_exponentiation(
                N1.clone(),
                BigInt::from(2).pow(i as u32) * j[i as usize],
                p.clone(),
            );
    }
    Ok(a1 * N2 % p)
}

/// Решение системы сравнений
pub fn solution_of_comparison_system<T: Into<BigInt>>(
    number_of_equations: usize,
    odds: Vec<T>,
    modules: Vec<T>,
) -> BigInt {
    let odds: Vec<BigInt> = odds.into_iter().map(|el| el.into()).collect();
    let modules: Vec<BigInt> = modules.into_iter().map(|el| el.into()).collect();

    let mut big_module: BigInt = BigInt::one();
    for el in &modules {
        big_module *= el;
    }

    let m_j = modules
        .iter()
        .map(|el| &big_module / el)
        .collect::<Vec<BigInt>>();
    let mut n_j: Vec<BigInt> = Vec::new();

    for i in 0..number_of_equations {
        n_j.push(expanded_euclidean_algorithm(m_j[i].clone(), modules[i].clone()).1);
    }

    let mut x: BigInt = BigInt::zero();

    for i in 0..number_of_equations {
        x += &odds[i] * &m_j[i] * &n_j[i];
    }

    x % big_module
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expanded_alg_euklid() {
        assert_eq!(
            (BigInt::from(3), BigInt::from(-5), BigInt::from(87)),
            expanded_euclidean_algorithm(678, 39)
        );
        assert_eq!(
            (BigInt::from(20), BigInt::from(0), BigInt::from(1)),
            expanded_euclidean_algorithm(20, 20)
        );
        assert_eq!(
            (BigInt::from(39), BigInt::from(0), BigInt::from(1)),
            expanded_euclidean_algorithm(0, 39)
        );
    }
    #[test]
    fn test_exponentiation() {
        assert_eq!(BigInt::from_str_radix("368063488259223267894700840060521865838338232037353204655959621437025609300472231530103873614505175218691345257589896391130393189447969771645832382192366076536631132001776175977932178658703660778465765811830827876982014124022948671975678131724958064427949902810498973271030787716781467419524180040734398996952930832508934116945966120176735120823151959779536852290090377452502236990839453416790640456116471139751546750048602189291028640970574762600185950226138244530187489211615864021135312077912018844630780307462205252807737757672094320692373101032517459518497524015120165166724189816766397247824175394802028228160027100623998873667435799073054618906855460488351426611310634023489044291860510352301912426608488807462312126590206830413782664554260411266378866626653755763627796569082931785645600816236891168141774993267488171702172191072731069216881668294625679492696148976999868715671440874206427212056717373099639711168901197440416590226524192782842896415414611688187391232048327738965820265934093108172054875188246591760877131657895633586576611857277011782497943522945011248430439201297015119468730712364007639373910811953430309476832453230123996750235710787086641070310288725389595138936784715274150426495416196669832679980253436807864187160054589045664027158817958549374490512399055448819148487049363674611664609890030088549591992466360050042566270348330911795487647045949301286614658650071299695652245266080672989921799342509291635330827874264789587306974472327718704306352445925996155619153783913237212716010410294999877569745287353422903443387562746452522860420416689019732913798073773281533570910205207767157128174184873357050830752777900041943256738499067821488421053870869022738698816059810579221002560882999884763252161747566893835178558961142349304466506402373556318707175710866983035313122068321102457824112014969387225476259342872866363550383840720010832906695360553556647545295849966279980830561242960013654529514995113584909050813015198928283202189194615501403435553060147713139766323195743324848047347575473228198492343231496580885057330510949058490527738662697480293583612233134502078182014347192522391449087738579081585795613547198599661273567662441490401862839817822686573112998663038868314974259766039340894024308383451039874674061160538242392803580758232755749310843694194787991556647907091849600704712003371103926967137408125713631396699343733288014254084819379380555174777020843568689927348949484201042595271932630685747613835385434424807024615161848223715989797178155169951121052285149157137697718850449708843330475301440373094611119631361702936342263219382793996895988331701890693689862459020775599439506870005130750427949747071390095256759203426671803377068109744629909769176319526837824364926844730545524646494321826241925107158040561607706364484910978348669388142016838792902926158979355432483611517588605967745393958061959024834251565197963477521095821435651996730128376734574843289089682710350244222290017891280419782767803785277960834729869249991658417000499998999", 10).unwrap(),
                  exponentiation(999,999));
        assert_eq!(BigInt::from(1), exponentiation(178, 0));
        assert_eq!(BigInt::from(16), exponentiation(2, 4));
    }

    #[test]
    fn test_rpn() {
        let mut rng = rand::thread_rng();
        let number = rpn(64, 100);
        let a = rng.gen_bigint_range(&BigInt::from(2), &BigInt::from(&number - 2));

        let result = match test_miller_rabin_for_prime_number::<BigInt>(number, a) {
            "Число n, вероятно, простое" => true,
            _ => false,
        };

        assert_eq!(true, result);
    }
}
