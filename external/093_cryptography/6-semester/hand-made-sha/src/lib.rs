pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut m = message.to_vec();
    m.push(0x80);
    if 64 - m.len() % 64 < 8 {
        m.append(&mut vec![0u8; 64 - m.len() % 64])
    }
    m.append(&mut vec![0u8; 64 - m.len() % 64 - 8]);
    m.append(&mut (message.len() as u64 * 8).to_be_bytes().to_vec());
    let blocks = m.chunks_exact(64);

    // 8 хэш-значений, которые представляют собой первые
    // 32 бита дробных частей квадратных корней из первых
    // восьми простых чисел
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    // 64 хэш-значений, которые представляют собой первые
    // 32 бита дробных частей кубических корней первых
    // 64 простых чисел
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    // Итерация по блокам данных
    for block in blocks {
        // Объединение байтов u8 в набор u32
        let mut w: Vec<u32> = block.chunks_exact(4).map(|chunk| {
            u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        }).collect();
        w.append(&mut vec![0u32; 48]);

        // Заполнение оставшихся 48 слов
        for i in 16..64 {
            let s0 = (w[i - 15].rotate_right(7)) ^ (w[i - 15].rotate_right(18)) ^ (w[i - 15] >> 3);
            let s1 = (w[i - 2].rotate_right(17)) ^ (w[i - 2].rotate_right(19)) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        // a, b, c, d, e, f, g, h
        // 0, 1, 2, 3, 4, 5, 6, 7
        let mut tmp_h: [u32; 8] = h.clone();

        // 64 раунда хэширования
        for i in 0..64 {
            let s1 = (tmp_h[4].rotate_right(6)) ^ (tmp_h[4].rotate_right(11)) ^ (tmp_h[4].rotate_right(25));
            let ch = (tmp_h[4] & tmp_h[5]) ^ (!tmp_h[4] & tmp_h[6]);
            let temp1 = tmp_h[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = (tmp_h[0].rotate_right(2)) ^ (tmp_h[0].rotate_right(13)) ^ (tmp_h[0].rotate_right(22));
            let maj = (tmp_h[0] & tmp_h[1]) ^ (tmp_h[0] & tmp_h[2]) ^ (tmp_h[1] & tmp_h[2]);
            let temp2 = s0.wrapping_add(maj);

            tmp_h[7] = tmp_h[6];
            tmp_h[6] = tmp_h[5];
            tmp_h[5] = tmp_h[4];
            tmp_h[4] = tmp_h[3].wrapping_add(temp1);
            tmp_h[3] = tmp_h[2];
            tmp_h[2] = tmp_h[1];
            tmp_h[1] = tmp_h[0];
            tmp_h[0] = temp1.wrapping_add(temp2);
        }

        for i in 0..8 {
            h[i] = h[i].wrapping_add(tmp_h[i]);
        }
    }

    h.map(|chunk| chunk.to_be_bytes()).concat().as_slice().try_into().unwrap()
}

pub fn sha512(message: &[u8]) -> [u8; 64] {
    let mut m = message.to_vec();
    m.push(0x80);
    if 128 - m.len() % 128 < 8 {
        m.append(&mut vec![0u8; 128 - m.len() % 128])
    }
    m.append(&mut vec![0u8; 128 - m.len() % 128 - 8]);
    m.append(&mut (message.len() as u64 * 8).to_be_bytes().to_vec());
    let blocks = m.chunks_exact(128);

    // 8 хэш-значений, которые представляют собой первые
    // 64 бита дробных частей квадратных корней из первых
    // восьми простых чисел
    let mut h: [u64; 8] = [
        0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
        0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
    ];
    // 80 хэш-значений, которые представляют собой первые
    // 64 бита дробных частей кубических корней первых
    // 80 простых чисел
    const K: [u64; 80] = [
        0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
        0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
        0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
        0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
        0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
        0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
        0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
        0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
        0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
        0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
        0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
        0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
        0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
        0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
        0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
        0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
        0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
        0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
        0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
        0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
    ];

    // Итерация по блокам данных
    for block in blocks {
        // Объединение байтов u8 в набор u64
        let mut w: Vec<u64> = block.chunks_exact(8).map(|chunk| {
            u64::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]])
        }).collect();
        w.append(&mut vec![0u64; 64]);

        // Заполнение оставшихся 80 слов
        for i in 16..80 {
            let s0 = (w[i - 15].rotate_right(1)) ^ (w[i - 15].rotate_right(8)) ^ (w[i - 15] >> 7);
            let s1 = (w[i - 2].rotate_right(19)) ^ (w[i - 2].rotate_right(61)) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        // a, b, c, d, e, f, g, h
        // 0, 1, 2, 3, 4, 5, 6, 7
        let mut tmp_h: [u64; 8] = h.clone();

        // 80 раундов хэширования
        for i in 0..80 {
            let s1 = (tmp_h[4].rotate_right(14)) ^ (tmp_h[4].rotate_right(18)) ^ (tmp_h[4].rotate_right(41));
            let ch = (tmp_h[4] & tmp_h[5]) ^ (!tmp_h[4] & tmp_h[6]);
            let temp1 = tmp_h[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = (tmp_h[0].rotate_right(28)) ^ (tmp_h[0].rotate_right(34)) ^ (tmp_h[0].rotate_right(39));
            let maj = (tmp_h[0] & tmp_h[1]) ^ (tmp_h[0] & tmp_h[2]) ^ (tmp_h[1] & tmp_h[2]);
            let temp2 = s0.wrapping_add(maj);

            tmp_h[7] = tmp_h[6];
            tmp_h[6] = tmp_h[5];
            tmp_h[5] = tmp_h[4];
            tmp_h[4] = tmp_h[3].wrapping_add(temp1);
            tmp_h[3] = tmp_h[2];
            tmp_h[2] = tmp_h[1];
            tmp_h[1] = tmp_h[0];
            tmp_h[0] = temp1.wrapping_add(temp2);
        }

        for i in 0..8 {
            h[i] = h[i].wrapping_add(tmp_h[i]);
        }
    }

    h.map(|chunk| chunk.to_be_bytes()).concat().as_slice().try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn sha256_text_test() -> Result<(), String> {
        let test_cases = [
            (
                "hello world\n",
                [
                    0xa9, 0x48, 0x90, 0x4f,
                    0x2f, 0x0f, 0x47, 0x9b,
                    0x8f, 0x81, 0x97, 0x69,
                    0x4b, 0x30, 0x18, 0x4b,
                    0x0d, 0x2e, 0xd1, 0xc1,
                    0xcd, 0x2a, 0x1e, 0xc0,
                    0xfb, 0x85, 0xd2, 0x99,
                    0xa1, 0x92, 0xa4, 0x47,
                ]
            ),
            (
                "привет мир\n",
                [
                    0x50, 0xd8, 0xc8, 0xf1,
                    0x54, 0x4f, 0xc2, 0x40,
                    0xa8, 0xde, 0x4f, 0x5d,
                    0x08, 0x9e, 0xc5, 0x35,
                    0x0c, 0x66, 0x76, 0xeb,
                    0x62, 0x59, 0x6f, 0xa4,
                    0x2b, 0x9e, 0xa2, 0xa6,
                    0x18, 0x06, 0x4a, 0x2b,
                ]
            ),
            (
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n",
                [
                    0x56, 0x29, 0x3a, 0x80,
                    0xe0, 0x39, 0x4d, 0x25,
                    0x2e, 0x99, 0x5f, 0x2d,
                    0xeb, 0xcc, 0xea, 0x82,
                    0x23, 0xe4, 0xb5, 0xb2,
                    0xb1, 0x50, 0xbe, 0xe2,
                    0x12, 0x72, 0x9b, 0x3b,
                    0x39, 0xac, 0x4d, 0x46,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let hash = sha256(input.as_bytes());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn sha256_video_test() -> Result<(), String> {
        let test_cases = [
            (
                Path::new("./assets/video.mp4"),
                [
                    0x08, 0x70, 0x5a, 0x38,
                    0x57, 0xcd, 0x1f, 0x3e,
                    0x51, 0xa5, 0x6c, 0x53,
                    0xec, 0xc6, 0x1e, 0x2e,
                    0xb2, 0x94, 0xf8, 0xe2,
                    0x19, 0xc9, 0x66, 0xde,
                    0x16, 0xb6, 0x38, 0xde,
                    0x4c, 0xe1, 0xd3, 0x00,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let data = fs::read(input).expect("Ошибка чтений файла");
            let hash = sha256(data.as_slice());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn sha512_text_test() -> Result<(), String> {
        let test_cases = [
            (
                "hello world\n",
                [
                    0xdb, 0x39, 0x74, 0xa9, 0x7f, 0x24, 0x07, 0xb7,
                    0xca, 0xe1, 0xae, 0x63, 0x7c, 0x00, 0x30, 0x68,
                    0x7a, 0x11, 0x91, 0x32, 0x74, 0xd5, 0x78, 0x49,
                    0x25, 0x58, 0xe3, 0x9c, 0x16, 0xc0, 0x17, 0xde,
                    0x84, 0xea, 0xcd, 0xc8, 0xc6, 0x2f, 0xe3, 0x4e,
                    0xe4, 0xe1, 0x2b, 0x4b, 0x14, 0x28, 0x81, 0x7f,
                    0x09, 0xb6, 0xa2, 0x76, 0x0c, 0x3f, 0x8a, 0x66,
                    0x4c, 0xea, 0xe9, 0x4d, 0x24, 0x34, 0xa5, 0x93,
                ]
            ),
            (
                "привет мир\n",
                [
                    0xb2, 0xa1, 0x45, 0x7f, 0x65, 0xf6, 0x49, 0x26,
                    0x25, 0xfe, 0x8e, 0x32, 0xe7, 0x6a, 0x27, 0xef,
                    0xa7, 0x08, 0x09, 0x48, 0x4d, 0x7b, 0x0e, 0x24,
                    0x16, 0xa7, 0xe6, 0xc3, 0x08, 0x5c, 0x33, 0x72,
                    0xe1, 0x5d, 0x1c, 0xc9, 0x15, 0x13, 0x66, 0xe2,
                    0xaf, 0x9d, 0x27, 0x92, 0xa1, 0xdb, 0x80, 0xfd,
                    0x53, 0x90, 0x1f, 0x3e, 0xae, 0x1d, 0x2a, 0x41,
                    0xbf, 0x9e, 0x80, 0x69, 0xc2, 0xdc, 0xc4, 0x9e,
                ]
            ),
            (
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n",
                [
                    0x0b, 0x7b, 0x28, 0xca, 0x2b, 0xf2, 0x8e, 0x25,
                    0x39, 0x29, 0xc8, 0xa2, 0x9d, 0xdb, 0x0a, 0xc2,
                    0xa3, 0x92, 0x26, 0xf8, 0x67, 0x02, 0xad, 0x1b,
                    0x1e, 0x51, 0x70, 0x3d, 0x5d, 0xce, 0xbd, 0x42,
                    0xaf, 0xf7, 0x74, 0x96, 0x9b, 0xb7, 0xe2, 0x3b,
                    0xf6, 0xc4, 0x39, 0xba, 0xb4, 0xea, 0xe3, 0x7c,
                    0xdf, 0xc8, 0x69, 0x78, 0xa1, 0x76, 0xc2, 0x7e,
                    0x83, 0x5c, 0xde, 0xf9, 0xc8, 0xaa, 0xf7, 0xde,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let hash = sha512(input.as_bytes());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn sha512_video_test() -> Result<(), String> {
        let test_cases = [
            (
                Path::new("./assets/video.mp4"),
                [
                    0x67, 0x6f, 0x5f, 0xe2, 0x1a, 0x36, 0xcc, 0x95,
                    0x4c, 0x29, 0x65, 0xeb, 0xb6, 0xb0, 0x83, 0xf4,
                    0x66, 0x32, 0x74, 0x18, 0x07, 0x94, 0x29, 0x28,
                    0x71, 0x5e, 0x0e, 0x75, 0xf6, 0x1c, 0x4d, 0xaa,
                    0xf1, 0x92, 0xbb, 0x83, 0x09, 0xc2, 0x5b, 0x0b,
                    0x0e, 0x5a, 0x6d, 0x29, 0x93, 0x06, 0x0b, 0xd8,
                    0x7b, 0xd2, 0x6d, 0xb0, 0x08, 0x16, 0xba, 0x4c,
                    0x9a, 0x77, 0x11, 0x36, 0x44, 0x04, 0xca, 0x71,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let data = fs::read(input).expect("Ошибка чтений файла");
            let hash = sha512(data.as_slice());
            assert_eq!(hash, expected);
        }

        Ok(())
    }
}