#![cfg(feature = "serde")]

use std::{collections::HashMap, hash::Hash};

use quickcheck_macros::quickcheck;
use serde::{Deserialize, Serialize};
use strumbra::{ArcString, BoxString, RcString};

#[derive(Serialize, Deserialize)]
struct Test<T: Eq + Hash> {
    raw: T,
    vec: Vec<T>,
    map: HashMap<T, T>,
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
fn serde(raw: String, vec: Vec<String>, map: HashMap<String, String>) {
    let wanted = Test { raw, vec, map };
    let json = serde_json::to_string(&wanted).unwrap();

    let boxed = serde_json::from_str::<Test<BoxString>>(&json).unwrap();
    let arc = serde_json::from_str::<Test<ArcString>>(&json).unwrap();
    let rc = serde_json::from_str::<Test<RcString>>(&json).unwrap();

    assert_eq!(wanted.raw, boxed.raw);
    assert_eq!(wanted.raw, arc.raw);
    assert_eq!(wanted.raw, rc.raw);

    assert_eq!(wanted.vec, boxed.vec);
    assert_eq!(wanted.vec, arc.vec);
    assert_eq!(wanted.vec, rc.vec);

    for (k, v) in wanted.map {
        let boxed_v = boxed.map.get(k.as_str()).expect("A existing value");
        let arc_v = arc.map.get(k.as_str()).expect("A existing value");
        let rc_v = rc.map.get(k.as_str()).expect("A existing value");

        assert_eq!(&v, boxed_v);
        assert_eq!(&v, arc_v);
        assert_eq!(&v, rc_v);
    }
}
