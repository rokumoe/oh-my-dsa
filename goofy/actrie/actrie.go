package trie

const (
	acnil  = -1
	acroot = 0
	acpren = 1
)

type acnode struct {
	ch   []byte
	next []int
	fail int
	end  int
}

func (n *acnode) addnode(c byte, x int) {
	n.ch = append(n.ch, c)
	n.next = append(n.next, x)
}

func (n *acnode) getnode(c byte) int {
	for i, ch := range n.ch {
		if ch == c {
			return n.next[i]
		}
	}
	return acnil
}

type ACTrie struct {
	nodes []acnode
	alloc int
}

func (t *ACTrie) Insert(s string) {
	nodes := t.nodes
	alloc := t.alloc
	if nodes == nil {
		nodes = make([]acnode, len(s)+acpren)
		alloc = acpren
		nodes[acroot].fail = acnil
	}
	p := acroot
	for i := 0; i < len(s); i++ {
		x := nodes[p].getnode(s[i])
		if x == acnil {
			if alloc == len(nodes) {
				t := make([]acnode, alloc+len(s)-i)
				copy(t, nodes)
				nodes = t
			}
			x = alloc
			alloc++
			nodes[p].addnode(s[i], x)
		}
		p = x
	}
	nodes[p].end++
	t.nodes = nodes
	t.alloc = alloc
}

func (t *ACTrie) Build() {
	nodes := t.nodes
	q := make([]int, 0, len(nodes[acroot].next))
	for _, p := range nodes[acroot].next {
		nodes[p].fail = acroot
		q = append(q, p)
	}
	for len(q) > 0 {
		p := q[0]
		q = q[1:]
		for i, c := range nodes[p].ch {
			x := nodes[p].next[i]
			fail := nodes[p].fail
			for fail != acnil {
				z := nodes[fail].getnode(c)
				if z != acnil {
					nodes[x].fail = z
					break
				}
				fail = nodes[fail].fail
			}
			if fail == acnil {
				nodes[x].fail = acroot
			}
			q = append(q, x)
		}
	}
}

func (t *ACTrie) Match(s string) int {
	nodes := t.nodes
	total := 0
	p := acroot
	for i := 0; i < len(s); i++ {
		x := nodes[p].getnode(s[i])
		for x == acnil && p != acroot {
			p = nodes[p].fail
			x = nodes[p].getnode(s[i])
		}
		p = x
		if p == acnil {
			p = acroot
		}
		x = p
		for x != acroot {
			total += nodes[x].end
			x = nodes[x].fail
		}
	}
	return total
}
