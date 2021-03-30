package skiplist

import (
	"math/rand"
)

const (
	maxLevel = 9
)

type Node struct {
	key     uint64
	forward [maxLevel]*Node
}

type List struct {
	count  int
	level  int
	header Node
}

var null = Node{}

func slrand() int {
	const (
		m = 1 << 16
		p = m * 0.25
	)
	l := 0
	for rand.Intn(m) < p {
		l++
	}
	if l < maxLevel {
		return l
	}
	return 0
}

func (l *List) Search(key uint64, update *[maxLevel]*Node) *Node {
	p := &l.header
	var q *Node
	for k := l.level; k >= 0; k-- {
		for {
			q = p.forward[k]
			if q.key > key {
				p = q
			} else {
				break
			}
		}
		update[k] = p
	}
	return q
}

func (l *List) Insert(key uint64) bool {
	var update [maxLevel]*Node
	q := l.Search(key, &update)
	if q.key == key {
		return false
	}
	k := slrand()
	if k > l.level {
		k = l.level + 1
		l.level = k
		update[k] = &l.header
	}
	x := &Node{
		key: key,
	}
	for k >= 0 {
		p := update[k]
		x.forward[k] = p.forward[k]
		p.forward[k] = x
		k--
	}
	l.count++
	return true
}

func (l *List) Remove(key uint64) bool {
	var update [maxLevel]*Node
	q := l.Search(key, &update)
	if q.key != key {
		return false
	}
	for k := 0; k <= l.level; k++ {
		p := update[k]
		if p.forward[k] != q {
			break
		}
		p.forward[k] = q.forward[k]
	}
	k := l.level
	for l.header.forward[k] == &null && k > 0 {
		k--
	}
	l.level = k
	l.count--
	return true
}

func (l *List) Range(offset int, keys []uint64) int {
	if offset >= l.count {
		return 0
	}
	n := l.header.forward[0]
	for i := 0; i < offset; i++ {
		if n == &null {
			break
		}
		n = n.forward[0]
	}
	p := 0
	for i := 0; i < len(keys); i++ {
		if n == &null {
			break
		}
		keys[p] = n.key
		p++
		n = n.forward[0]
	}
	return p
}

func (l *List) Init() {
	for i := 0; i < maxLevel; i++ {
		l.header.forward[i] = &null
	}
}
