#[rustler::nif]
pub fn example_add(a: i64, b: i64) -> i64 {
    a + b
}