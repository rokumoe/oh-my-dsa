use crate::sort;

fn find_mid(a: &mut [i32], l: usize, r: usize) -> usize {
    if l == r {
        return l;
    }
    let mut n = 0;
    let mut i = l;
    while i + 5 < r {
        sort::insert_sort(a, i, i + 4);
        n = i - l;
        a.swap(l + n / 5, i + 2);
        i += 5;
    }
    let num = r - i + 1;
    if num > 0 {
        sort::insert_sort(a, i, i + num - 1);
        n = i - l;
        a.swap(l + n / 5, i + num / 2);
    }
    n /= 5;
    if n == l {
        l
    } else {
        find_mid(a, l, l + n)
    }
}

fn partition(a: &mut [i32], l: usize, r: usize, p: usize) -> usize {
    a.swap(p, l);
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

// https://zhuanlan.zhihu.com/p/31498036
pub fn bfprt(a: &mut [i32], l: usize, r: usize, k: usize) -> i32 {
    let p = find_mid(a, l, r);
    let i = partition(a, l, r, p);
    let m = i - l + 1;
    if m == k {
        a[i]
    } else if m > k {
        bfprt(a, l, i - 1, k)
    } else {
        bfprt(a, i + 1, r, k - m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfprt() {
        let mut a = vec![72, 6, 57, 88, 60, 42, 83, 73, 48, 85, 10, 14, 23];
        let n = a.len() - 1;
        bfprt(&mut a, 0, n, 6);
        sort::insert_sort(&mut a, 0, 5);
        assert_eq!(&a[..6], &[6, 10, 14, 23, 42, 48]);
    }
}
