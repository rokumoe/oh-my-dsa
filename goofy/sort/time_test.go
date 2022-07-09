package pdqsort

import (
	"sort"
	"testing"
	"time"
)

const (
	timeTestNum = 100000
	timeTestMod = 1000000
)

func TestStdSliceSortTime(t *testing.T) {
	a := generateData(timeTestNum, timeTestMod)
	start := time.Now()
	sort.Slice(a, func(i, j int) bool { return a[i] < a[j] })
	cost := time.Since(start)
	t.Log("cost:", cost)
}

func TestStdSortTime(t *testing.T) {
	a := generateData(timeTestNum, timeTestMod)
	start := time.Now()
	sort.Ints(a)
	cost := time.Since(start)
	t.Log("cost:", cost)
}

func TestShellSortTime(t *testing.T) {
	a := generateData(timeTestNum, timeTestMod)
	start := time.Now()
	ShellSort(a)
	cost := time.Since(start)
	t.Log("cost:", cost)
}

func TestPdqsortTime(t *testing.T) {
	a := generateData(timeTestNum, timeTestMod)
	start := time.Now()
	Pdqsort(a)
	cost := time.Since(start)
	t.Log("cost:", cost)
}
