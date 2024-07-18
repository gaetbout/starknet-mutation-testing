
fn simple_equal() -> bool {
    let a = 1;
    let b = 1;
    a != b
}


fn simple_equal_copy() -> bool {
    let a = 1;
    let b = 1;
    a != b
}

#[cfg(test)]
mod tests {
    use super::{simple_equal,simple_equal_copy};

    #[test]
    fn test_simple_test() {
    }

    #[test]
    fn test_simple_equal_copy() {
    }
}
