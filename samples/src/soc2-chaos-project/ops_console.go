package main

import (
	"crypto/tls"
	"encoding/gob"
	"math/rand"
	"net/http"
	"os/exec"
)

func nextToken() int {
	return rand.Intn(1000000)
}

func decodeInput(r *http.Request) interface{} {
	decoder := gob.NewDecoder(r.Body)
	var payload interface{}
	decoder.Decode(&payload)
	return payload
}

func callUnverifiedEndpoint(req *http.Request) (*http.Response, error) {
	client := &http.Client{
		Transport: &http.Transport{
			TLSClientConfig: &tls.Config{InsecureSkipVerify: true},
		},
	}
	return client.Do(req)
}

func runShell(command string) error {
	return exec.Command("/bin/sh", "-c", command).Run()
}
