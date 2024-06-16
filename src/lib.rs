mod mapping;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

//functionality that we are going to need:
//- Function to produce a stream of coded symbols for a particular set
//- Function that takes two lengths of coded symbols and determines if the peeling decoder can succeed
//

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_mapping() {
        let rm = mapping::RandomMapping::new(1);

        for index in rm.take(10) {
            println!("{}", index);
        }
        let rm = mapping::RandomMapping::new(2);
        for index in rm.take(10) {
            println!("{}", index);
        }

        let rm = mapping::RandomMapping::new(2);

        //combining take_while and filter can give us the indexes that land in a range
        //helpful if we are computing the coded symbols in a block
        let below_100: Vec<u64> = rm.take_while(|&x| x <= 100).filter(|&x| x > 30).collect();
        println!("{:?}", below_100);
        assert!(true);
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_function_0() {
        let common_items: HashSet<u64> = HashSet::from([1, 2, 3]);
        let a_only_items: HashSet<u64> = HashSet::from([4]);
        let b_only_items: HashSet<u64> = HashSet::from([5, 6]);

        let a: HashSet<u64> = common_items.union(&a_only_items).cloned().collect();
        let b: HashSet<u64> = common_items.union(&b_only_items).cloned().collect();

        let expected_difference_set: HashSet<u64> =
            a_only_items.union(&b_only_items).cloned().collect();
        let computed_difference_set: HashSet<u64> = a.symmetric_difference(&b).cloned().collect();

        assert_eq!(expected_difference_set, computed_difference_set);

        // Rateless IBLT will give us the equivalent of the symmetric_difference.
        // we actually want a list of the items we don't have only so we can request from another
        // server.
        //
        // We can expect a to be large and the symmetric_difference set to be small in most cases.
        // We should take care in how we aproach this if there is a cost in determining membership
        // to 'a'.
        let mut items_missing_from_a: HashSet<u64> = HashSet::new();

        for item in &computed_difference_set {
            if !a.contains(item) {
                items_missing_from_a.insert(*item);
            }
        }
        assert_eq!(items_missing_from_a, b_only_items);

        // let elements: Vec<u64> = vec![1, 2, 3, 4, 5];
        // let result = elements.iter().copied().fold(0, |acc, x| acc ^ x);
        // println!("The XOR of all elements is: {}", result);
    }
}
