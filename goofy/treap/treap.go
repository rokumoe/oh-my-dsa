package treap

type Node struct {
	key   int
	prior int
	left  *Node
	right *Node
}

func zig(p *Node) *Node {
	right := p.right
	p.right = right.left
	right.left = p
	return right
}

func zag(p *Node) *Node {
	left := p.left
	p.left = left.right
	left.right = p
	return left
}

func Insert(p *Node, key int, prior int) *Node {
	if p == nil {
		p = &Node{
			key:   key,
			prior: prior,
		}
	} else if key < p.key {
		p.left = Insert(p.left, key, prior)
		if p.left.prior < p.prior {
			p = zag(p)
		}
	} else {
		p.right = Insert(p.right, key, prior)
		if p.right.prior < p.prior {
			p = zig(p)
		}
	}
	return p
}

func Remove(p *Node, key int) *Node {
	if p == nil {
		return nil
	}
	if key < p.key {
		p.left = Remove(p.left, key)
	} else if key > p.key {
		p.right = Remove(p.right, key)
	} else if p.left == nil {
		p = p.right
	} else if p.right == nil {
		p = p.left
	} else if p.left.prior < p.right.prior {
		p = zag(p)
		p.right = Remove(p.right, key)
	} else {
		p = zig(p)
		p.left = Remove(p.left, key)
	}
	return p
}

func Search(p *Node, key int) *Node {
	if p != nil {
		if key < p.key {
			p = Search(p.left, key)
		} else if key > p.key {
			p = Search(p.right, key)
		}
	}
	return p
}
