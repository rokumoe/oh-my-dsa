package rbtree

import (
	"fmt"
	"io"
	"strings"
)

func printNode(w io.Writer, n *Node, indent int, d int) {
	tag := [...]string{"|<", "|>", "*"}
	if n != nil {
		data := fmt.Sprintf(": %d", n.key)
		color := "black"
		if n.red {
			color = "red"
		}
		_, _ = fmt.Fprintf(w, "%s%s <%p ^%p>[%s]%s\n", strings.Repeat(" ", indent*2), tag[d], n, n.parent, color, data)
		for i, c := range n.link {
			printNode(w, c, indent+1, i)
		}
	} else {
		_, _ = fmt.Fprintf(w, "%s%s NIL\n", strings.Repeat(" ", indent*2), tag[d])
	}
}

func PrintTree(w io.Writer, t *Tree) {
	printNode(w, t.root, 0, 2)
}
