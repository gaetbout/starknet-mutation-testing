fn simple(a: u64) -> bool {
    let b = 10;
    a < b
}


#[cfg(test)]
mod tests {
    use super::simple;

    #[test]
    fn test() {
        assert(simple(9), 'pass');
    }

    #[test]
    fn test_3() {
        assert(!simple(10), 'pass');
    }
}
