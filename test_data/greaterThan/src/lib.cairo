fn simple_1(a: u64) -> bool {
    let b = 1;
    a > b
}


fn simple_2(a: u64) -> bool {
    let b = 1;
    a > b
}

#[cfg(test)]
mod tests {
    use super::{simple_1,simple_2};

    #[test]
    fn test_1() {
        assert(simple_1(2), 'pass');
    }

    #[test]
    fn test_2() {
        assert(simple_2(2), 'pass');
    }

    #[test]
    fn test_3() {
        assert(!simple_1(1), 'pass');
    }

    #[test]
    fn test_4() {
        assert(!simple_2(1), 'pass');
    }
}
