
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_function_0() {
        let a : HashSet<u64> = HashSet::from([1, 2, 3]);
        let b : HashSet<u64> = HashSet::from([4, 2, 3, 4]);

        // Print 1, 4 in arbitrary order.
        for x in a.symmetric_difference(&b) {
            println!("{x}");
        }

        let diff1: HashSet<_> = a.symmetric_difference(&b).collect();
        let diff2: HashSet<_> = b.symmetric_difference(&a).collect();

        assert_eq!(diff1, diff2);
        assert_eq!(diff1, [1, 4].iter().collect());

        // let elements: Vec<u64> = vec![1, 2, 3, 4, 5];
        // let result = elements.iter().copied().fold(0, |acc, x| acc ^ x);
        // println!("The XOR of all elements is: {}", result);
    }
}
