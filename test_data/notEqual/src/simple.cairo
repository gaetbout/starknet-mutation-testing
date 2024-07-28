
fn simple_1() -> bool {
    let a = 1;
    let b = 1;
    a != b
}


fn simple_2() -> bool {
    let a = 1;
    let b = 1;
    a != b
}

#[cfg(test)]
mod tests {
    use super::{simple_1,simple_2};

    #[test]
    fn test_1() {
        assert(!simple_1(), 'pass');
    }

    #[test]
    fn test_2() {
        assert(!simple_2(), 'pass');
    }
}
