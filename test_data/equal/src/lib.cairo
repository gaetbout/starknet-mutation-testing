fn simple() -> bool {
    let a = 1;
    let b = 1;
    a == b
}

#[cfg(test)]
mod tests {
    use super::simple;

    #[test]
    fn test() {
        assert(simple(), 'pass');
    }

}
