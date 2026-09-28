//! The translator's output compiles in the kernel and gives the method's
//! answers. The golden file is written by the translator from the worked
//! example (crates/vleo-sheet/tests/the_translator_writes_the_example.rs);
//! this runs it where every node's translated method runs.

#[path = "golden/example_method.rs"]
mod example_method;

use vleo_core::physics::method::MethodError;

#[test]
fn the_translated_example_answers_and_refuses_as_its_method_does() {
    let v = example_method::evaluate(6_628_137.0).expect("250 km answers");
    assert!((v - 7754.84549737).abs() / 7754.84549737 < 1e-9, "{v}");
    assert!(matches!(
        example_method::evaluate(6_000_000.0),
        Err(MethodError::Refused("the orbit is inside the Earth"))
    ));
}

#[path = "golden/every_construct.rs"]
mod every_construct;

/// Every construct of the method language, translated, gives the interpreter's
/// answer to the last bit — or refuses, or finds the mathematics undefined,
/// exactly where the interpreter does.
#[test]
fn every_construct_gives_the_interpreters_answer_to_the_bit() {
    let expect = include_str!("golden/every_construct.expect");
    let mut n = 0;
    for line in expect
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let (x, want) = line.split_once(' ').unwrap();
        let x: f64 = x.parse().unwrap();
        let got = every_construct::evaluate(x);
        match want {
            "refused" => assert!(
                matches!(got, Err(MethodError::Refused(_))),
                "at {x}: {got:?}"
            ),
            "degenerate" => assert!(
                matches!(got, Err(MethodError::Degenerate { .. })),
                "at {x}: {got:?}"
            ),
            bits => assert_eq!(
                got.map(f64::to_bits),
                Ok(u64::from_str_radix(bits, 16).unwrap()),
                "at {x}"
            ),
        }
        n += 1;
    }
    assert!(n >= 10, "the expectations file is nearly empty");
}
