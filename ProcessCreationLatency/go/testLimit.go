package main

import (
    "fmt"
)

var ch = make(chan byte)

func f() {
    <-ch // Block this goroutine
}

func main() {
    i := 1
    for ; ; i++ {
        go f()
        fmt.Printf("Number of goroutines: %d\n", i)
    }

}

