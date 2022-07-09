package pdqsort

import (
	"sort"
	"testing"
)

func TestPdqsort(t *testing.T) {
	a := generateData(100000, 1000000)
	Pdqsort(a)
	if !sort.IntsAreSorted(a) {
		t.Fatal(a)
	}
}
