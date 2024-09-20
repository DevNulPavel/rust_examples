use std::{cmp, thread};

use quickcheck_macros::quickcheck;
use strumbra::{ArcString, BoxString, RcString};

#[test]
fn size_of() {
    assert_eq!(16, std::mem::size_of::<BoxString>());
    assert_eq!(16, std::mem::size_of::<ArcString>());
}

#[test]
fn construct_inlined() {
    let boxed = BoxString::try_from("hello world").expect("A valid Umbra-style string");
    let arc = ArcString::try_from("hello world").expect("A valid Umbra-style string");
    let rc = RcString::try_from("hello world").expect("A valid Umbra-style string");
    assert_eq!("hello world", boxed.as_ref());
    assert_eq!("hello world", arc.as_ref());
    assert_eq!("hello world", rc.as_ref());
}

#[test]
fn construct_allocated() {
    let boxed = BoxString::try_from("Good Morning, Vietnam").expect("A valid Umbra-style string");
    let arc = ArcString::try_from("Good Morning, Vietnam").expect("A valid Umbra-style string");
    let rc = RcString::try_from("Good Morning, Vietnam").expect("A valid Umbra-style string");
    assert_eq!("Good Morning, Vietnam", boxed.as_ref());
    assert_eq!("Good Morning, Vietnam", arc.as_ref());
    assert_eq!("Good Morning, Vietnam", rc.as_ref());
}

#[test]
fn move_across_threads_inlined() {
    let boxed = BoxString::try_from("hello world").expect("A valid Umbra-style string");
    let arc = ArcString::try_from("hello world").expect("A valid Umbra-style string");

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let cloned0 = boxed.clone();
            let cloned1 = arc.clone();
            thread::spawn(move || assert_eq!(cloned0, cloned1))
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread finishes successfully");
    }
}

#[test]
fn move_across_threads_allocated() {
    let boxed = BoxString::try_from("Good Morning, Vietnam").expect("A valid Umbra-style string");
    let arc = ArcString::try_from("Good Morning, Vietnam").expect("A valid Umbra-style string");

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let cloned0 = boxed.clone();
            let cloned1 = arc.clone();
            thread::spawn(move || assert_eq!(cloned0, cloned1))
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread finishes successfully");
    }
}

#[test]
fn eq_string_different_length_with_null_byte() {
    let assert = |lhs: &str, rhs: &str| {
        let boxed_lhs = BoxString::try_from(lhs).expect("A valid Umbra-style string");
        let boxed_rhs = BoxString::try_from(rhs).expect("A valid Umbra-style string");
        let arc_lhs = ArcString::try_from(lhs).expect("A valid Umbra-style string");
        let arc_rhs = ArcString::try_from(rhs).expect("A valid Umbra-style string");
        let rc_lhs = RcString::try_from(lhs).expect("A valid Umbra-style string");
        let rc_rhs = RcString::try_from(rhs).expect("A valid Umbra-style string");

        assert_ne!(boxed_lhs, boxed_rhs);
        assert_ne!(boxed_lhs, arc_rhs);
        assert_ne!(boxed_lhs, rc_rhs);

        assert_ne!(arc_lhs, arc_rhs);
        assert_ne!(arc_lhs, rc_rhs);
        assert_ne!(arc_lhs, boxed_rhs);

        assert_ne!(rc_lhs, rc_rhs);
        assert_ne!(rc_lhs, boxed_rhs);
        assert_ne!(rc_lhs, arc_rhs);
    };

    assert("abc", "abc\0");
    assert("abc\0", "abc");
    assert("abcdefghijk", "abcdefghijk\0");
    assert("abcdefghijk\0", "abcdefghijk");
}

#[test]
fn cmp_string_different_length_with_null_byte() {
    let assert = |lhs: &str, rhs: &str| {
        let boxed_lhs = BoxString::try_from(lhs).expect("A valid Umbra-style string");
        let boxed_rhs = BoxString::try_from(rhs).expect("A valid Umbra-style string");
        let arc_lhs = ArcString::try_from(lhs).expect("A valid Umbra-style string");
        let arc_rhs = ArcString::try_from(rhs).expect("A valid Umbra-style string");
        let rc_lhs = RcString::try_from(lhs).expect("A valid Umbra-style string");
        let rc_rhs = RcString::try_from(rhs).expect("A valid Umbra-style string");

        assert_eq!(Ord::cmp(lhs, rhs), Ord::cmp(&boxed_lhs, &boxed_rhs));
        assert_eq!(Ord::cmp(lhs, rhs), Ord::cmp(&arc_lhs, &arc_rhs));
        assert_eq!(Ord::cmp(lhs, rhs), Ord::cmp(&rc_lhs, &rc_rhs));

        assert_eq!(
            PartialOrd::partial_cmp(lhs, rhs),
            PartialOrd::partial_cmp(&boxed_lhs, &arc_rhs)
        );
        assert_eq!(
            PartialOrd::partial_cmp(lhs, rhs),
            PartialOrd::partial_cmp(&boxed_lhs, &rc_rhs)
        );

        assert_eq!(
            PartialOrd::partial_cmp(lhs, rhs),
            PartialOrd::partial_cmp(&arc_lhs, &rc_rhs)
        );
        assert_eq!(
            PartialOrd::partial_cmp(lhs, rhs),
            PartialOrd::partial_cmp(&arc_lhs, &boxed_rhs)
        );

        assert_eq!(
            PartialOrd::partial_cmp(lhs, rhs),
            PartialOrd::partial_cmp(&rc_lhs, &boxed_rhs)
        );
        assert_eq!(
            PartialOrd::partial_cmp(lhs, rhs),
            PartialOrd::partial_cmp(&rc_lhs, &arc_rhs)
        );
    };

    assert("abc", "abc\0");
    assert("abc\0", "abc");
    assert("abcdefghijk", "abcdefghijk\0");
    assert("abcdefghijk\0", "abcdefghijk");
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn format_debug(s: String) {
    let expected = format!("{s:?}");
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    assert_eq!(expected, format!("{boxed:?}"));
    assert_eq!(expected, format!("{arc:?}"));
    assert_eq!(expected, format!("{rc:?}"));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn format_display(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    assert_eq!(s, format!("{boxed}"));
    assert_eq!(s, format!("{arc}"));
    assert_eq!(s, format!("{rc}"));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn eq(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");

    assert_eq!(boxed, boxed);
    assert_eq!(boxed, arc);
    assert_eq!(boxed, rc);

    assert_eq!(arc, arc);
    assert_eq!(arc, rc);
    assert_eq!(arc, boxed);

    assert_eq!(rc, rc);
    assert_eq!(rc, boxed);
    assert_eq!(rc, arc);
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn eq_str(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    assert_eq!(s.as_str(), &boxed);
    assert_eq!(s.as_str(), &arc);
    assert_eq!(s.as_str(), &rc);
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn eq_string(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    assert_eq!(s, boxed);
    assert_eq!(s, arc);
    assert_eq!(s, rc);
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn ne(s1: String, s2: String) {
    let lhs_boxed = BoxString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_boxed = BoxString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_arc = ArcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_arc = ArcString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_rc = RcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_rc = RcString::try_from(s2.as_str()).expect("A valid Umbra-style string");

    assert_ne!(lhs_boxed, rhs_boxed);
    assert_ne!(lhs_boxed, rhs_arc);
    assert_ne!(lhs_boxed, rhs_rc);

    assert_ne!(lhs_arc, rhs_arc);
    assert_ne!(lhs_arc, rhs_rc);
    assert_ne!(lhs_arc, rhs_boxed);

    assert_ne!(lhs_rc, rhs_rc);
    assert_ne!(lhs_rc, rhs_boxed);
    assert_ne!(lhs_rc, rhs_arc);
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn ne_str(s1: String, s2: String) {
    let lhs_boxed = BoxString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_boxed = BoxString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_arc = ArcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_arc = ArcString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_rc = RcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_rc = RcString::try_from(s2.as_str()).expect("A valid Umbra-style string");

    assert_ne!(s1.as_str(), &rhs_boxed);
    assert_ne!(s1.as_str(), &rhs_arc);
    assert_ne!(s1.as_str(), &rhs_rc);

    assert_ne!(&lhs_boxed, s2.as_str());
    assert_ne!(&lhs_arc, s2.as_str());
    assert_ne!(&lhs_rc, s2.as_str());
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn ne_string(s1: String, s2: String) {
    let lhs_boxed = BoxString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_boxed = BoxString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_arc = ArcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_arc = ArcString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_rc = RcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_rc = RcString::try_from(s2.as_str()).expect("A valid Umbra-style string");

    assert_ne!(s1, rhs_boxed);
    assert_ne!(s1, rhs_arc);
    assert_ne!(s1, rhs_rc);

    assert_ne!(lhs_boxed, s2);
    assert_ne!(lhs_arc, s2);
    assert_ne!(lhs_rc, s2);
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn cmp_same(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");

    assert_eq!(cmp::Ordering::Equal, boxed.cmp(&boxed));
    assert_eq!(cmp::Ordering::Equal, arc.cmp(&arc));
    assert_eq!(cmp::Ordering::Equal, rc.cmp(&rc));

    assert_eq!(Some(cmp::Ordering::Equal), boxed.partial_cmp(&arc));
    assert_eq!(Some(cmp::Ordering::Equal), boxed.partial_cmp(&rc));

    assert_eq!(Some(cmp::Ordering::Equal), arc.partial_cmp(&boxed));
    assert_eq!(Some(cmp::Ordering::Equal), arc.partial_cmp(&rc));

    assert_eq!(Some(cmp::Ordering::Equal), rc.partial_cmp(&boxed));
    assert_eq!(Some(cmp::Ordering::Equal), rc.partial_cmp(&arc));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn cmp_same_str(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");

    assert_eq!(Some(cmp::Ordering::Equal), boxed.partial_cmp(s.as_str()));
    assert_eq!(Some(cmp::Ordering::Equal), arc.partial_cmp(s.as_str()));
    assert_eq!(Some(cmp::Ordering::Equal), rc.partial_cmp(s.as_str()));

    assert_eq!(Some(cmp::Ordering::Equal), s.as_str().partial_cmp(&boxed));
    assert_eq!(Some(cmp::Ordering::Equal), s.as_str().partial_cmp(&arc));
    assert_eq!(Some(cmp::Ordering::Equal), s.as_str().partial_cmp(&rc));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn cmp_same_string(s: String) {
    let boxed = BoxString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let arc = ArcString::try_from(s.as_str()).expect("A valid Umbra-style string");
    let rc = RcString::try_from(s.as_str()).expect("A valid Umbra-style string");

    assert_eq!(Some(cmp::Ordering::Equal), boxed.partial_cmp(&s));
    assert_eq!(Some(cmp::Ordering::Equal), arc.partial_cmp(&s));
    assert_eq!(Some(cmp::Ordering::Equal), rc.partial_cmp(&s));

    assert_eq!(Some(cmp::Ordering::Equal), s.partial_cmp(&boxed));
    assert_eq!(Some(cmp::Ordering::Equal), s.partial_cmp(&arc));
    assert_eq!(Some(cmp::Ordering::Equal), s.partial_cmp(&rc));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn cmp_diff(s1: String, s2: String) {
    let lhs_boxed = BoxString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_boxed = BoxString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_arc = ArcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_arc = ArcString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_rc = RcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_rc = RcString::try_from(s2.as_str()).expect("A valid Umbra-style string");

    assert_eq!(s1.cmp(&s2), lhs_boxed.cmp(&rhs_boxed));
    assert_eq!(s1.cmp(&s2), lhs_arc.cmp(&rhs_arc));
    assert_eq!(s1.cmp(&s2), lhs_rc.cmp(&rhs_rc));

    assert_eq!(s1.partial_cmp(&s2), lhs_boxed.partial_cmp(&rhs_arc));
    assert_eq!(s1.partial_cmp(&s2), lhs_boxed.partial_cmp(&rhs_rc));

    assert_eq!(s1.partial_cmp(&s2), lhs_arc.partial_cmp(&rhs_boxed));
    assert_eq!(s1.partial_cmp(&s2), lhs_arc.partial_cmp(&rhs_rc));

    assert_eq!(s1.partial_cmp(&s2), lhs_rc.partial_cmp(&rhs_boxed));
    assert_eq!(s1.partial_cmp(&s2), lhs_rc.partial_cmp(&rhs_arc));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn cmp_diff_str(s1: String, s2: String) {
    let lhs_boxed = BoxString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_boxed = BoxString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_arc = ArcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_arc = ArcString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_rc = RcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_rc = RcString::try_from(s2.as_str()).expect("A valid Umbra-style string");

    assert_eq!(s1.partial_cmp(&s2), s1.as_str().partial_cmp(&rhs_boxed));
    assert_eq!(s1.partial_cmp(&s2), s1.as_str().partial_cmp(&rhs_arc));
    assert_eq!(s1.partial_cmp(&s2), s1.as_str().partial_cmp(&rhs_rc));

    assert_eq!(s1.partial_cmp(&s2), lhs_boxed.partial_cmp(s2.as_str()));
    assert_eq!(s1.partial_cmp(&s2), lhs_arc.partial_cmp(s2.as_str()));
    assert_eq!(s1.partial_cmp(&s2), lhs_rc.partial_cmp(s2.as_str()));
}

#[quickcheck]
#[cfg_attr(miri, ignore)]
#[allow(clippy::needless_pass_by_value)]
fn cmp_diff_string(s1: String, s2: String) {
    let lhs_boxed = BoxString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_boxed = BoxString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_arc = ArcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_arc = ArcString::try_from(s2.as_str()).expect("A valid Umbra-style string");
    let lhs_rc = RcString::try_from(s1.as_str()).expect("A valid Umbra-style string");
    let rhs_rc = RcString::try_from(s2.as_str()).expect("A valid Umbra-style string");

    assert_eq!(s1.partial_cmp(&s2), s1.partial_cmp(&rhs_boxed));
    assert_eq!(s1.partial_cmp(&s2), s1.partial_cmp(&rhs_arc));
    assert_eq!(s1.partial_cmp(&s2), s1.partial_cmp(&rhs_rc));

    assert_eq!(s1.partial_cmp(&s2), lhs_boxed.partial_cmp(&s2));
    assert_eq!(s1.partial_cmp(&s2), lhs_arc.partial_cmp(&s2));
    assert_eq!(s1.partial_cmp(&s2), lhs_rc.partial_cmp(&s2));
}
