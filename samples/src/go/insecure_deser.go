package main

import (
	"encoding/gob"
	"net/http"
	"os"
)

func deserializeFromRequest(r *http.Request) interface{} {
	// Unsafe: deserializing untrusted request body with gob
	var obj interface{}
	dec := gob.NewDecoder(r.Body)
	dec.Decode(&obj)
	return obj
}
