fn partition(a: &mut [i32], l: usize, r: usize) -> usize {
    let mut i = l;
    let mut j = r;
    let pivot = a[l];
    while i < j {
        while a[j] >= pivot && i < j {
            j -= 1;
        }
        a[i] = a[j];
        while a[i] <= pivot && i < j {
            i += 1;
        }
        a[j] = a[i];
    }
    a[i] = pivot;
    i
}

pub fn quick_sort(a: &mut [i32], l: usize, r: usize) {
    if l < r {
        let k = partition(a, l, r);
        if k > 0 {
            quick_sort(a, l, k - 1);
        }
        quick_sort(a, k + 1, r);
    }
}

pub fn insert_sort(a: &mut [i32], l: usize, r: usize) {
    for i in l + 1..=r {
        if a[i - 1] > a[i] {
            let t = a[i];
            let mut j = i;
            while j > l && a[j - 1] > t {
                a[j] = a[j - 1];
                j -= 1;
            }
            a[j] = t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_sort() {
        let mut a = vec![72, 6, 57, 88, 60, 42, 83, 73, 48, 85];
        let n = a.len() - 1;
        quick_sort(&mut a, 0, n);
        assert_eq!(a, vec![6, 42, 48, 57, 60, 72, 73, 83, 85, 88]);
    }

    #[test]
    fn test_insert_sort() {
        let mut a = vec![72, 6, 57, 88, 60, 42, 83, 73, 48, 85];
        let n = a.len() - 1;
        insert_sort(&mut a, 0, n);
        assert_eq!(a, vec![6, 42, 48, 57, 60, 72, 73, 83, 85, 88]);
    }
}
