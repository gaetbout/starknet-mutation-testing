fn simple() -> bool {
    assert(false, 'fail');
    true
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
