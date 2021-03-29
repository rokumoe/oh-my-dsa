package trie

import (
	"testing"
)

func TestACTrieMatch(t *testing.T) {
	tr := ACTrie{}
	tr.Insert("say")
	tr.Insert("she")
	tr.Insert("he")
	tr.Insert("shr")
	tr.Insert("her")
	tr.Build()
	t.Log("yasherhs", tr.Match("yasherhs"))
}

func BenchmarkACTrieMatch(b *testing.B) {
	tr := ACTrie{}
	tr.Insert("say")
	tr.Insert("she")
	tr.Insert("he")
	tr.Insert("shr")
	tr.Insert("her")
	tr.Build()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		tr.Match("yasherhs")
	}
}
