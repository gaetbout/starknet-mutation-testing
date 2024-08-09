use core::num::traits::Zero;

fn simple() {
    assert(!0.is_zero(), 'fail');
}

#[cfg(test)]
mod tests {
    use super::simple;

    #[test]
    #[should_panic(expected: ('fail',))]
    fn test() {
        simple();
    }
}
