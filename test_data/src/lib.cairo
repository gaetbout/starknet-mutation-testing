
fn simple_equal() -> bool {
    let a = 1;
    let b = 1;
    a == b
}

#[cfg(test)]
mod tests {
    use super::simple_equal;

    #[test]
    fn simple_test() {
        assert(simple_equal(), 'pass');
    }
}
