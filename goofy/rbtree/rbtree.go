package rbtree

type Node struct {
	link   [2]*Node
	parent *Node
	key    int
	red    bool
}

type Tree struct {
	root *Node
}

func (t *Tree) searchNode(key int) *Node {
	n := t.root
	for n != nil {
		if n.key > key {
			n = n.link[0]
		} else if n.key < key {
			n = n.link[1]
		} else {
			return n
		}
	}
	return nil
}

func (t *Tree) Search(key int) bool {
	return t.searchNode(key) != nil
}

func (t *Tree) fixInsert(n *Node) {
	for {
		p := n.parent
		if p == nil || !p.red {
			break
		}
		g := p.parent
		if g == nil {
			break
		}

		var dir int
		if g.link[0] != p {
			dir = 1
		} else {
			dir = 0
		}
		dir &= 1
		sib := (1 - dir) & 1

		u := g.link[sib]
		if u != nil && u.red {
			p.red = false
			u.red = false
			g.red = true
			n = g
		} else {
			if p.link[sib] == n {
				p.link[sib] = n.link[dir]
				n.link[dir] = p
				g.link[dir] = n
				p.parent = n
				if p.link[sib] != nil {
					p.link[sib].parent = p
				}
				p = n
			}
			gg := g.parent
			g.link[dir] = p.link[sib]
			p.link[sib] = g
			p.parent = g.parent
			g.parent = p
			if g.link[dir] != nil {
				g.link[dir].parent = g
			}
			p.red = false
			g.red = true
			if gg == nil {
				t.root = p
			} else if gg.link[0] == g {
				gg.link[0] = p
			} else {
				gg.link[1] = p
			}
			return
		}
	}
	t.root.red = false
}

func (t *Tree) Insert(key int) bool {
	var (
		p   *Node
		dir int
	)
	n := t.root
	for n != nil {
		if key < n.key {
			dir = 0
		} else if key > n.key {
			dir = 1
		} else {
			return true
		}
		p = n
		n = n.link[dir&1]
	}
	n = &Node{
		parent: p,
		key:    key,
		red:    true,
	}
	if p != nil {
		p.link[dir&1] = n
	} else {
		t.root = n
	}
	t.fixInsert(n)
	return false
}

func (t *Tree) fixDelete(p *Node, dir int, root *Node) {
	for {
		dir &= 1
		sib := (1 - dir) & 1

		x := p.link[dir]
		if x != nil && x.red {
			x.red = false
			break
		}
		if p == root {
			break
		}

		g := p.parent
		if g == nil {
			g = root
		}

		w := p.link[sib]
		if w.red {
			w.red = false
			p.red = true

			p.link[sib] = w.link[dir]
			w.link[dir] = p
			if g.link[0] == p {
				g.link[0] = w
			} else {
				g.link[1] = w
			}

			w.parent = p.parent
			p.parent = w

			g = w
			w = p.link[sib]
			w.parent = p
		}

		if (w.link[0] == nil || !w.link[0].red) && (w.link[1] == nil || !w.link[1].red) {
			w.red = true
		} else {
			if w.link[sib] == nil || !w.link[sib].red {
				y := w.link[dir]
				y.red = false
				w.red = true
				w.link[dir] = y.link[sib]
				y.link[sib] = w
				if w.link[dir] != nil {
					w.link[dir].parent = w
				}
				p.link[sib] = y
				w = y
				w.link[sib].parent = w
			}

			w.red = p.red
			p.red = false
			w.link[sib].red = false

			p.link[sib] = w.link[dir]
			w.link[dir] = p
			if g.link[0] == p {
				g.link[0] = w
			} else {
				g.link[1] = w
			}

			w.parent = p.parent
			p.parent = w
			if p.link[sib] != nil {
				p.link[sib].parent = p
			}

			break
		}
		z := p
		p = p.parent
		if p == nil {
			p = root
		}
		if p.link[0] == z {
			dir = 0
		} else {
			dir = 1
		}
	}
}

func (t *Tree) Delete(key int) bool {
	n := t.searchNode(key)
	if n == nil {
		return false
	}

	root := Node{link: [2]*Node{0: t.root}}
	p := n.parent
	dir := 0
	if p == nil {
		p = &root
	} else if p.link[1] == n {
		dir = 1
	}

	if n.link[1] == nil {
		c := n.link[0]
		p.link[dir] = c
		if c != nil {
			c.parent = n.parent
		}
	} else {
		r := n.link[1]
		if r.link[0] == nil {
			r.link[0] = n.link[0]
			p.link[dir] = r
			r.parent = n.parent
			if r.link[0] != nil {
				r.link[0].parent = r
			}

			t := n.red
			n.red = r.red
			r.red = t

			p = r
			dir = 1
		} else {
			s := r.link[0]
			for s.link[0] != nil {
				s = s.link[0]
			}
			r = s.parent
			r.link[0] = s.link[1]
			s.link[0] = n.link[0]
			s.link[1] = n.link[1]
			p.link[dir] = s
			if s.link[0] != nil {
				s.link[0].parent = s
			}
			s.link[1].parent = s
			s.parent = n.parent
			if r.link[0] != nil {
				r.link[0].parent = r
			}

			t := n.red
			n.red = s.red
			s.red = t

			p = r
			dir = 0
		}
	}

	if !n.red {
		t.fixDelete(p, dir, &root)
	}

	t.root = root.link[0]

	return true
}
