package rbtree

import (
	"math/rand"
	"testing"
	"time"
)

func init() {
	rand.Seed(time.Now().UnixNano())
}

func TestInsert(t *testing.T) {
	tr := &RBTree[int, int]{Cmp: func(a, b int) int { return a - b }}
	for i := 0; i < 100; i++ {
		n := rand.Int() % 100
		t.Log("> insert", n)
		t.Log(tr.Insert(n, i))
	}
}

func TestDelete(t *testing.T) {
	nums := make([]int, 100)
	for i := range nums {
		nums[i] = rand.Int() % 100
	}
	tr := &RBTree[int, int]{Cmp: func(a, b int) int { return a - b }}
	for i, n := range nums {
		t.Log("> insert", n, tr.Insert(n, i))
	}
	for _, n := range nums {
		t.Log("> delete", n)
		t.Log(tr.Delete(n))
	}
	t.Log("root nil", tr.root == nil)
}
