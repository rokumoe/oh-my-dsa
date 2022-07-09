package pdqsort

import (
	"math/bits"
	"unsafe"
)

func xorshift(seed uint32) uint32 {
	seed ^= seed << 13
	seed ^= seed >> 17
	seed ^= seed << 5
	return seed
}

func shiftBack[T Ord](a []T, j int) {
	for {
		swap(a, j, j+1)
		j++
		if j == len(a)-1 || a[j+1] >= a[j] {
			break
		}
	}
}

func partialInsertionSort[T Ord](a []T) bool {
	const (
		maxSteps         = 5
		shortestShifting = 50
	)
	i := 1
	for j := 0; j < maxSteps; j++ {
		for i < len(a) && a[i] >= a[i-1] {
			i++
		}
		if i == len(a) {
			return true
		}
		if len(a) < shortestShifting {
			return false
		}
		shiftForward(a, i)
		if i+1 < len(a) && a[i+1] < a[i] {
			shiftBack(a, i)
		}
	}
	return false
}

func partitionInBlocks[T Ord](a []T, pivotElem *T) int {
	const blockSize = 128
	var (
		lstart int
		lend   int
		rstart int
		rend   int

		loffs [blockSize]byte
		roffs [blockSize]byte
	)
	l := 0
	r := len(a)
	lblock := blockSize
	rblock := blockSize
	for {
		isDone := (r - l) <= 2*blockSize
		if isDone {
			rem := r - l
			if lstart < lend || rstart < rend {
				rem -= blockSize
			}

			if lstart < lend {
				rblock = rem
			} else if rstart < rend {
				lblock = rem
			} else {
				lblock = rem / 2
				rblock = rem - lblock
			}
		}

		if lstart == lend {
			lstart = 0
			lend = lstart
			elem := l
			for i := 0; i < lblock; i++ {
				loffs[lend] = byte(i)
				if a[elem] >= *pivotElem {
					lend++
				}
				elem++
			}
		}

		if rstart == rend {
			rstart = 0
			rend = rstart
			elem := r
			for i := 0; i < rblock; i++ {
				elem--
				roffs[rend] = byte(i)
				if a[elem] < *pivotElem {
					rend++
				}
			}
		}

		count := min(lend-lstart, rend-rstart)
		if count > 0 {
			tmp := a[l+int(loffs[lstart])]
			a[l+int(loffs[lstart])] = a[r-int(roffs[rstart])-1]
			for i := 1; i < count; i++ {
				lstart++
				a[r-int(roffs[rstart])-1] = a[l+int(loffs[lstart])]
				rstart++
				a[l+int(loffs[lstart])] = a[r-int(roffs[rstart])-1]
			}
			a[r-int(roffs[rstart])-1] = tmp
			lstart++
			rstart++
		}
		if lstart == lend {
			l += lblock
		}
		if rstart == rend {
			r -= rblock
		}
		if isDone {
			break
		}
	}

	if lstart < lend {
		for lstart < lend {
			lend--
			swap(a, l+int(loffs[lend]), r-1)
			r--
		}
		return r
	} else if rstart < rend {
		for rstart < rend {
			rend--
			swap(a, l, r-int(roffs[rend])-1)
			l++
		}
	}
	return l
}

func partition[T Ord](a []T, pivot int) (int, bool) {
	swap(a, 0, pivot)
	pivotElem := &a[0]
	pa := a[1:]
	l := 0
	r := len(pa)
	for l < r && pa[l] < *pivotElem {
		l++
	}
	for l < r && pa[r-1] >= *pivotElem {
		r--
	}
	mid := l + partitionInBlocks(pa[l:r], pivotElem)
	wasPartitioned := l >= r
	swap(a, 0, mid)
	return mid, wasPartitioned
}

func partitionEqual[T Ord](a []T, pivot int) int {
	swap(a, 0, pivot)
	pivotElem := a[0]
	a = a[1:]
	l := 0
	r := len(a)
	for {
		for l < r && pivotElem >= a[l] {
			l++
		}
		for l < r && pivotElem < a[r-1] {
			r--
		}
		if l != r {
			break
		}
		r--
		swap(a, l, r)
		l++
	}
	return l + 1
}

func breakPatterns[T any](a []T) {
	if len(a) >= 8 {
		seed := uint32(len(a))
		mask := uint((1 << bits.Len(uint(len(a)))) - 1)
		pos := len(a) / 4 * 2
		for i := -1; i <= 1; i++ {
			var rnd uint
			if bits.UintSize == 64 {
				seed = xorshift(seed)
				rnd1 := uint(seed)
				seed = xorshift(seed)
				rnd2 := uint(seed)
				rnd = rnd1<<32 | rnd2
			} else {
				seed = xorshift(seed)
				rnd = uint(seed)
			}
			other := int(rnd & mask)
			if other >= len(a) {
				other -= len(a)
			}
			swap(a, pos+i, other)
		}
	}
}

func sort2[T Ord](a []T, i int, j int, n int) (int, int, int) {
	if a[j] < a[i] {
		return j, i, n + 1
	} else {
		return i, j, n
	}
}

func sort3[T Ord](a []T, i int, j int, k int, n int) (int, int) {
	i, j, n = sort2(a, i, j, n)
	j, k, n = sort2(a, j, k, n)
	i, j, n = sort2(a, i, j, n)
	return j, n
}

func sortAdjacent[T Ord](a []T, m int, n int) (int, int) {
	return sort3(a, m-1, m, m+1, n)
}

func choosePivot[T Ord](a []T) (int, bool) {
	const (
		shortestMedianOfMedians = 50
		maxSwaps                = 4 * 3
	)

	i := len(a) / 4 * 1
	j := len(a) / 4 * 2
	k := len(a) / 4 * 3

	swaps := 0

	if len(a) >= 8 {
		if len(a) >= shortestMedianOfMedians {
			i, swaps = sortAdjacent(a, i, swaps)
			j, swaps = sortAdjacent(a, j, swaps)
			k, swaps = sortAdjacent(a, k, swaps)
		}
		j, swaps = sort3(a, i, j, k, swaps)
	}

	if swaps < maxSwaps {
		return j, swaps == 0
	} else {
		rev(a)
		return len(a) - 1 - j, true
	}
}

func recurse[T Ord](a []T, pred *T, limit int) {
	const maxInsertion = 20
	wasBalanced := true
	wasPartitioned := true
	for {
		if len(a) < maxInsertion {
			InsertionSort(a)
			return
		}
		if limit == 0 {
			HeapSort(a)
			return
		}

		if !wasBalanced {
			breakPatterns(a)
			limit--
		}

		pivot, likelySorted := choosePivot(a)

		if wasBalanced && wasPartitioned && likelySorted {
			if partialInsertionSort(a) {
				return
			}
		}

		if pred != nil && *pred >= a[pivot] {
			mid := partitionEqual(a, pivot)
			a = a[mid:]
			continue
		}

		midIdx, partitioned := partition(a, pivot)
		wasBalanced = min(midIdx, len(a)-midIdx) >= len(a)/8
		wasPartitioned = partitioned

		left, pivotElem, right := a[:midIdx], &a[midIdx], a[midIdx+1:]
		if len(left) < len(right) {
			recurse(left, pred, limit)
			a = right
			pred = pivotElem
		} else {
			recurse(right, pivotElem, limit)
			a = left
		}
	}
}

func Pdqsort[T Ord](a []T) {
	if len(a) != 0 && unsafe.Sizeof(*(*T)(nil)) != 0 {
		recurse(a, nil, bits.Len(uint(len(a))))
	}
}
