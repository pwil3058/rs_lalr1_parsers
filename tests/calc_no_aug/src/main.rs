#[macro_use]
extern crate lazy_static;

mod calc_no_aug;

use lalr1::Parser;

fn main() {
    let mut calc = calc_no_aug::Calc::new();
    calc.parse_text("a = 1 + 8 * 5".to_string(), String::new())
        .unwrap();
    assert_eq!(calc.variable("a"), Some(41.0));
    calc.parse_text("b = (1 + 8) * 5".to_string(), String::new())
        .unwrap();
    assert_eq!(calc.variable("b"), Some(45.0));
    calc.parse_text("c = a + b".to_string(), String::new())
        .unwrap();
    assert_eq!(calc.variable("c"), Some(86.0));
    calc.parse_text("a + b + c".to_string(), String::new())
        .unwrap();
    println!("Hello, world! No crashes!!!");
}
