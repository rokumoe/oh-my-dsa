package pdqsort

type Ord interface {
	~int | ~uint | ~string
}

func shiftForward[T Ord](a []T, j int) {
	for {
		swap(a, j, j-1)
		j--
		if j == 0 || a[j] >= a[j-1] {
			break
		}
	}
}

func InsertionSort[T Ord](a []T) {
	for i := 1; i < len(a); i++ {
		if a[i] < a[i-1] {
			shiftForward(a, i)
		}
	}
}

func shiftForwardStep[T Ord](a []T, j int, h int) {
	for {
		swap(a, j, j-h)
		j -= h
		if j < h || a[j] >= a[j-h] {
			break
		}
	}
}

func ShellSort[T Ord](a []T) {
	h := 1
	for h < len(a)/3 {
		h = h*3 + 1
	}
	for ; h >= 1; h /= 3 {
		for i := h; i < len(a); i++ {
			if a[i] < a[i-h] {
				shiftForwardStep(a, i, h)
			}
		}
	}
}

func siftDown[T Ord](a []T, i int) {
	n := len(a) / 2
	for i < n {
		left := 2*i + 1
		right := 2*i + 2
		if right < len(a) && a[left] < a[right] {
			left = right
		}
		if left >= len(a) || a[i] >= a[left] {
			break
		}
		swap(a, i, left)
		i = left
	}
}

func HeapSort[T Ord](a []T) {
	for i := len(a)/2 - 1; i >= 0; i-- {
		siftDown(a, i)
	}
	for i := len(a) - 1; i >= 1; i-- {
		swap(a, 0, i)
		siftDown(a[:i], 0)
	}
}
