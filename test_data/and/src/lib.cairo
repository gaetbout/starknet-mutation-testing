fn simple(a: bool) {
    assert(a && true, 'fail');
}

#[cfg(test)]
mod tests {
    use super::simple;

    #[test]
    #[should_panic(expected: ('fail',))]
    fn test() {
        simple(false);
    }
}
