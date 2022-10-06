package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"
)

type ValueType struct {
	Value int `json:"value"`
	State int `json:"state"`
}

type Proposer struct {
	mu        sync.Mutex
	ballotNum int
}

type Acceptor struct {
	mu        sync.Mutex
	ballotNum int
	vballot   int
	value     *ValueType
}

var self struct {
	proposer  Proposer
	acceptor  Acceptor
	id        int
	acceptors []string
}

type Prepare struct {
	BallotNum int `json:"ballot_num"`
}

type Promise struct {
	OK        bool       `json:"ok"`
	BallotNum int        `json:"ballot_num"`
	Value     *ValueType `json:"value"`
}

func onPrepare(args *Prepare) *Promise {
	acceptor := &self.acceptor
	acceptor.mu.Lock()
	defer acceptor.mu.Unlock()
	if acceptor.ballotNum > args.BallotNum {
		return &Promise{
			OK: false,
		}
	}
	acceptor.ballotNum = args.BallotNum
	return &Promise{
		OK:        true,
		BallotNum: acceptor.vballot,
		Value:     acceptor.value,
	}
}

type Propose struct {
	BallotNum int        `json:"ballot_num"`
	Value     *ValueType `json:"value"`
}

type Accept struct {
	OK bool `json:"ok"`
}

func onAccept(args *Propose) *Accept {
	acceptor := &self.acceptor
	acceptor.mu.Lock()
	defer acceptor.mu.Unlock()
	if acceptor.ballotNum > args.BallotNum {
		return &Accept{
			OK: false,
		}
	}
	acceptor.ballotNum = args.BallotNum
	acceptor.vballot = args.BallotNum
	acceptor.value = args.Value
	return &Accept{
		OK: true,
	}
}

func invoke(node int, method string, args interface{}, reply interface{}) error {
	data, _ := json.Marshal(args)
	resp, err := http.Post(self.acceptors[node]+method, "application/json", bytes.NewReader(data))
	if err != nil {
		return err
	}
	respBody, _ := ioutil.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("%d %s", resp.StatusCode, respBody)
	}
	resp.Body.Close()
	return json.Unmarshal(respBody, reply)
}

func nextBallotNum() int {
	proposer := &self.proposer
	proposer.mu.Lock()
	proposer.ballotNum++
	num := proposer.ballotNum
	proposer.mu.Unlock()
	return num*100 + self.id
}

func prepare(ballotNum int) (bool, int, *ValueType) {
	replys := make(chan *Promise, len(self.acceptors))
	for i := range self.acceptors {
		go func(i int) {
			args := &Prepare{
				BallotNum: ballotNum,
			}
			var reply Promise
			err := invoke(i, "/paxos/prepare", args, &reply)
			if err != nil {
				log.Printf("prepare %d: %v", i, err)
				replys <- &Promise{
					OK: false,
				}
				return
			}
			log.Printf("prepare %d: %+v", i, reply)
			replys <- &reply
		}(i)
	}
	var value *ValueType
	maxBallotNum := ballotNum
	promised := 0
	n := len(self.acceptors)
	for i := 0; i < n; i++ {
		reply := <-replys
		if !reply.OK {
			continue
		}
		promised++
		if reply.Value != nil {
			if value == nil {
				maxBallotNum = reply.BallotNum
				value = reply.Value
			} else if reply.BallotNum > maxBallotNum {
				maxBallotNum = reply.BallotNum
				value = reply.Value
			}
		}
	}
	if promised < len(self.acceptors)/2+1 {
		return false, 0, nil
	}
	return true, maxBallotNum, value
}

func accept(ballotNum int, value *ValueType) bool {
	replys := make(chan *Accept, len(self.acceptors))
	for i := range self.acceptors {
		go func(i int) {
			args := &Propose{
				BallotNum: ballotNum,
				Value:     value,
			}
			var reply Accept
			err := invoke(i, "/paxos/accept", args, &reply)
			if err != nil {
				log.Printf("accept %d: %v", i, err)
				replys <- &Accept{
					OK: false,
				}
				return
			}
			log.Printf("accept %d: %+v", i, reply)
			replys <- &reply
		}(i)
	}

	accepted := 0
	n := len(self.acceptors)
	for i := 0; i < n; i++ {
		reply := <-replys
		if !reply.OK {
			continue
		}
		accepted++
	}
	return accepted >= len(self.acceptors)/2+1
}

func apply(state int, val int) {
	log.Printf("apply: %d %d", state, val)
}

func caspaxos(state int, val int) (bool, int) {
	ballotNum := nextBallotNum()
	ok, ballotNum, current := prepare(ballotNum)
	if !ok {
		return false, 0
	}
	var next *ValueType
	if current == nil {
		apply(0, val)
		next = &ValueType{
			Value: val,
			State: 0,
		}
	} else if state == current.State {
		apply(current.State+1, val)
		next = &ValueType{
			Value: val,
			State: current.State + 1,
		}
	} else {
		return false, current.State
	}
	ok = accept(ballotNum, next)
	return ok, next.State
}

func main() {
	var (
		id     int
		listen string
		nodes  string
	)
	flag.IntVar(&id, "i", os.Getpid()%100, "id")
	flag.StringVar(&listen, "l", ":8000", "listen")
	flag.StringVar(&nodes, "n", "", "nodes")
	flag.Parse()

	self.id = id
	if nodes != "" {
		self.acceptors = strings.Split(nodes, ",")
	}
	http.HandleFunc("/paxos/prepare", func(w http.ResponseWriter, r *http.Request) {
		reqData, _ := ioutil.ReadAll(r.Body)
		var args Prepare
		err := json.Unmarshal(reqData, &args)
		if err != nil {
			http.Error(w, http.StatusText(http.StatusBadRequest), http.StatusBadRequest)
			return
		}
		reply := onPrepare(&args)
		replyData, _ := json.Marshal(reply)
		w.Write(replyData)
	})
	http.HandleFunc("/paxos/accept", func(w http.ResponseWriter, r *http.Request) {
		reqData, _ := ioutil.ReadAll(r.Body)
		var args Propose
		err := json.Unmarshal(reqData, &args)
		if err != nil {
			http.Error(w, http.StatusText(http.StatusBadRequest), http.StatusBadRequest)
			return
		}
		reply := onAccept(&args)
		replyData, _ := json.Marshal(reply)
		w.Write(replyData)
	})
	http.HandleFunc("/submit", func(w http.ResponseWriter, r *http.Request) {
		state, _ := strconv.Atoi(r.FormValue("state"))
		val, _ := strconv.Atoi(r.FormValue("val"))
		ok, state := caspaxos(state, val)
		fmt.Fprintf(w, "%t %d\n", ok, state)
	})
	log.Fatalf("http: %v", http.ListenAndServe(listen, nil))
}
