package pdqsort

import (
	"math/rand"
	"sort"
	"testing"
)

func generateData(n int, m int) []int {
	rnd := rand.New(rand.NewSource(0))
	a := make([]int, n)
	for i := range a {
		a[i] = rnd.Intn(m)
	}
	return a
}

func TestInsertionSort(t *testing.T) {
	a := generateData(1000, 1000)
	InsertionSort(a)
	if !sort.IntsAreSorted(a) {
		t.Fatal(a)
	}
}

func TestHeapSort(t *testing.T) {
	a := generateData(1000, 1000)
	HeapSort(a)
	if !sort.IntsAreSorted(a) {
		t.Fatal(a)
	}
}

func TestShellSort(t *testing.T) {
	a := generateData(1000, 1000)
	ShellSort(a)
	if !sort.IntsAreSorted(a) {
		t.Fatal(a)
	}
}
