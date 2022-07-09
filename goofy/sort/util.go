package pdqsort

func swap[T any](a []T, i, j int) {
	t := a[i]
	a[i] = a[j]
	a[j] = t
}

func rev[T any](a []T) {
	l := 0
	r := len(a) - 1
	for l < r {
		swap(a, l, r)
		l++
		r--
	}
}

func min[T int](a, b T) T {
	if a < b {
		return a
	} else {
		return b
	}
}
