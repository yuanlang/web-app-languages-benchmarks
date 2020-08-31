package main

import (
    "fmt"
    "os"
    "time"
)

var ch = make(chan byte)

func f() {
    <-ch // Block this goroutine
}

func main() {
    i := 1
    file, err := os.Create("test_go_limit.txt")
    if err != nil {
        return
    }

    for ; ; i++ {
        go f()
        now := time.Now()
        sec := now.Unix()
        fmt.Fprintf(file, "ts(sec): %d goroutine no: %d\n", sec, i)
    }

}

