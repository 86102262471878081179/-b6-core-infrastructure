// main.go
package main

import (
	"crypto/sha512"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

type WormBlock struct {
	Block     int    `json:"block"`
	Hash      string `json:"hash"`
	Timestamp string `json:"timestamp"`
}

var (
	lastHash = "850b2ef003f53db90f704b16e54869fc6ba6265eae13aafde9046fe0e4d99dab"
	blockNum = 0
	mu       sync.Mutex
	upgrader = websocket.Upgrader{CheckOrigin: func(r *http.Request) bool { return true }}
)

func main() {
	http.HandleFunc("/api/v1/worm/status", handleStatus)
	http.HandleFunc("/ws/telemetry", handleTelemetry)
	fmt.Println("🚀 Go Blue Core läuft auf Port 8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}

func handleStatus(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "GO_CORE_LIVE", 
		"last_hash": lastHash, 
		"block": blockNum,
	})
}

func handleTelemetry(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Print("upgrade failed: ", err)
		return
	}
	defer conn.Close()

	for {
		mu.Lock()
		blockNum++
		h := sha512.Sum512([]byte(fmt.Sprintf("%s%d", lastHash, time.Now().UnixNano())))
		lastHash = hex.EncodeToString(h[:])
		currentBlock := blockNum
		mu.Unlock()

		msg := WormBlock{
			Block:     currentBlock,
			Hash:      lastHash[:16],
			Timestamp: time.Now().Format(time.RFC3339),
		}
		conn.WriteJSON(msg)
		time.Sleep(1500 * time.Millisecond)
	}
}
