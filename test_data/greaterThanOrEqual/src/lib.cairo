fn simple(a: u64) -> bool {
    let b = 1;
    a >= b
}

#[cfg(test)]
mod tests {
    use super::simple;

    #[test]
    fn test() {
        assert(simple(1), 'pass');
        assert(simple(2), 'pass');
    }
}
