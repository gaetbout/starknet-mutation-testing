
fn simple(a: u64) -> bool {
    let b = 2;
    a <= b
}

#[cfg(test)]
mod tests {
    use super::simple;

    #[test]
    fn test() {
        assert(simple(2), 'pass');
        assert(simple(1), 'pass');
    }
}
