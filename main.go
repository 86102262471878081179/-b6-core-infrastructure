// main.go
// ═══════════════════════════════════════════════════════════════════════════════
// YGGDRASIL_OS :: GO BLUE CORE (v26.1.0)
// Vollständige Go-Implementierung des L.S.S. Neuro-Core Backends.
// ═══════════════════════════════════════════════════════════════════════════════
package main

import (
	"crypto/sha256"
	"crypto/sha512"
	"encoding/hex"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"log"
	"math/rand"
	"net/http"
	"os"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

// ═══════════════════════════════════════════════════════════════════════════════
// 1. KONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

type Config struct {
	Port          string
	ExpectedSeal  string
	MasterSeal    string
	LedgerCap     int
	DecayRate     float64
	MaxEntropy    float64
	AllowedOrigin string
}

var cfg Config

func loadConfig() {
	cfg = Config{
		Port:          getEnv("PORT", "8080"),
		ExpectedSeal:  getEnv("EXPECTED_SEAL", "Φ_SOVEREIGN_ROOT_SEAL_EX_TUNC"),
		MasterSeal:    getEnv("MASTER_SEAL", "850b2ef003f53db90f704b16e54869fc6ba6265eae13aafde9046fe0e4d99dab"),
		LedgerCap:     50,
		DecayRate:     0.08,
		MaxEntropy:    5.0,
		AllowedOrigin: getEnv("ALLOWED_ORIGINS", "*"),
	}
}

func getEnv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. TYPES
// ═══════════════════════════════════════════════════════════════════════════════

type WormBlock struct {
	Block     int    `json:"block"`
	Hash      string `json:"hash"`
	PrevHash  string `json:"prev_hash"`
	Event     string `json:"event"`
	Timestamp string `json:"timestamp"`
}

type Inmate struct {
	ID     string `json:"id"`
	Token  string `json:"token"`
	Reason string `json:"reason"`
	TS     int64  `json:"ts"`
}

type ApexRequest struct {
	Message  string `json:"message"`
	IsThreat bool   `json:"is_threat"`
	AuthSeal string `json:"auth_seal"`
}

type BrummelRequest struct {
	Message  string `json:"message"`
	AuthSeal string `json:"auth_seal"`
}

type ApexResponse struct {
	Status         string                 `json:"status"`
	BioState       string                 `json:"bio_state"`
	KortexMode     string                 `json:"kortex_mode"`
	ThreatLevel    float64                `json:"threat_level"`
	SignatureHits  []string               `json:"signature_hits,omitempty"`
	GothamInmate   string                 `json:"gotham_inmate,omitempty"`
	Wusulum        map[string]interface{} `json:"wusulum,omitempty"`
	FoucaultResult string                 `json:"foucault_result,omitempty"`
	XJustizOutput  string                 `json:"xjustiz_output,omitempty"`
	AsmeSeal       string                 `json:"asme_seal,omitempty"`
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. WORM-LEDGER (Thread-Safe, Hash-Chain)
// ═══════════════════════════════════════════════════════════════════════════════

type Ledger struct {
	mu      sync.Mutex
	blocks  []WormBlock
	last    string
	started time.Time
}

var ledger = &Ledger{
	last:    getEnv("MASTER_SEAL", "850b2ef0"),
	started: time.Now(),
}

func (l *Ledger) Append(event, payload string) WormBlock {
	l.mu.Lock()
	defer l.mu.Unlock()

	next := len(l.blocks) + 1
	material := fmt.Sprintf("%s|%s|%s|%d", l.last, event, payload, time.Now().UnixNano())
	h := sha512.Sum512([]byte(material))
	newHash := hex.EncodeToString(h[:])

	block := WormBlock{
		Block:     next,
		Hash:      newHash,
		PrevHash:  l.last,
		Event:     event,
		Timestamp: time.Now().Format(time.RFC3339),
	}

	l.blocks = append(l.blocks, block)
	if len(l.blocks) > cfg.LedgerCap {
		l.blocks = l.blocks[len(l.blocks)-cfg.LedgerCap:]
	}
	l.last = newHash

	return block
}

func (l *Ledger) Tail(n int) []WormBlock {
	l.mu.Lock()
	defer l.mu.Unlock()
	if n > len(l.blocks) {
		n = len(l.blocks)
	}
	out := make([]WormBlock, n)
	copy(out, l.blocks[len(l.blocks)-n:])
	return out
}

func (l *Ledger) Count() int {
	l.mu.Lock()
	defer l.mu.Unlock()
	// Zähle alle bisherigen Blöcke (nicht nur Ring-Buffer)
	if len(l.blocks) == 0 {
		return 0
	}
	return l.blocks[len(l.blocks)-1].Block
}

func (l *Ledger) LastHash() string {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.last
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. GOTHAM CITY (Arkham Cache)
// ═══════════════════════════════════════════════════════════════════════════════

type Gotham struct {
	mu      sync.Mutex
	inmates []Inmate
}

var gotham = &Gotham{}

func (g *Gotham) Incarcerate(token, reason string) string {
	g.mu.Lock()
	defer g.mu.Unlock()
	h := sha256.Sum256([]byte(token))
	id := hex.EncodeToString(h[:])[:8]
	g.inmates = append(g.inmates, Inmate{
		ID: id, Token: token, Reason: reason, TS: time.Now().Unix(),
	})
	log.Printf("[GOTHAM] Inmate %s eingesperrt: %s", id, reason[:min(60, len(reason))])
	return id
}

func (g *Gotham) Count() int {
	g.mu.Lock()
	defer g.mu.Unlock()
	return len(g.inmates)
}

func (g *Gotham) List() []Inmate {
	g.mu.Lock()
	defer g.mu.Unlock()
	out := make([]Inmate, len(g.inmates))
	copy(out, g.inmates)
	return out
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. L.S.S. STATE (Wusulum, Polyvagal, Brummel, Foucault)
// ═══════════════════════════════════════════════════════════════════════════════

type Polyvagal struct {
	mu     sync.Mutex
	stress float64
	state  string
}

var polyvagal = &Polyvagal{state: "VENTRAL_VAGAL"}

func (p *Polyvagal) Update(threat float64) string {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.stress = maxF(0.0, p.stress-cfg.DecayRate)
	p.stress = minF(1.0, p.stress+threat)
	if p.stress < 0.4 {
		p.state = "VENTRAL_VAGAL"
		return "KREATIV_MODUS (Wusulum aktiv)"
	} else if p.stress < 0.8 {
		p.state = "SYMPATHIKUS"
		return "KAMPF_MODUS (Kortex aktiviert KrakenGuard)"
	}
	p.state = "DORSAL_VAGAL"
	return "SHUTDOWN_MODUS (Fail-Closed)"
}

func (p *Polyvagal) State() string {
	p.mu.Lock()
	defer p.mu.Unlock()
	return p.state
}

func (p *Polyvagal) CoRegulate(amount float64) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.stress = maxF(0.0, p.stress-amount)
	if p.stress < 0.4 {
		p.state = "VENTRAL_VAGAL"
	}
}

type Wusulum struct {
	mu      sync.Mutex
	state   string
	memory  []string
	entropy float64
}

var wusulum = &Wusulum{state: "GENESIS_WAHRHEIT"}

func (w *Wusulum) Cycle() map[string]interface{} {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.entropy = minF(cfg.MaxEntropy, w.entropy+0.1)
	attn := rand.Float64()
	material := fmt.Sprintf("%f:%s:%d", attn, w.state, len(w.memory))
	h := sha256.Sum256([]byte(material))
	newState := fmt.Sprintf("TOKEN_%s", hex.EncodeToString(h[:])[:8])
	if len(newState) > 20 {
		newState = newState[:20]
	}
	w.memory = append(w.memory, newState)
	w.state = newState
	mh := sha512.Sum512([]byte(strings.Join(w.memory, ",")))
	return map[string]interface{}{
		"state":     w.state,
		"attention": attn,
		"entropy":   w.entropy,
		"worm_hash": hex.EncodeToString(mh[:])[:32],
	}
}

func (w *Wusulum) Entropy() float64 {
	w.mu.Lock()
	defer w.mu.Unlock()
	return w.entropy
}

type Brummel struct {
	mu    sync.Mutex
	phase int
}

var brummel = &Brummel{}

func (b *Brummel) Oscillate() int {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.phase = (b.phase + 1) % 18
	diff := b.phase - 9
	if diff < 0 {
		diff = -diff
	}
	return 9 - diff
}

func (b *Brummel) Phase() int {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.phase
}

func FoucaultInvert(token string) string {
	h := sha256.Sum256([]byte(token))
	if h[0] > 127 {
		return fmt.Sprintf("SYS_CONFORM_%s", hex.EncodeToString(h[:])[:6])
	}
	return token
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. GREMMEL SIGNATURE SCANNER
// ═══════════════════════════════════════════════════════════════════════════════

var signatures = []string{
	"pissingerkeule", "mettwurst", "rhabarberbarbara",
	"gremmel", "cargo-kult", "urheberinfo", "§ 839 bgb",
	"brummel", "dummelt",
}

func ScanSignatures(text string) (bool, []string) {
	lower := strings.ToLower(text)
	var hits []string
	for _, sig := range signatures {
		if strings.Contains(lower, sig) {
			hits = append(hits, sig)
		}
	}
	return len(hits) > 0, hits
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. XJUSTIZ XML GENERATOR
// ═══════════════════════════════════════════════════════════════════════════════

type XJustizNachricht struct {
	XMLName xml.Name `xml:"nachricht.straf.0300"`
	Xmlns   string   `xml:"xmlns,attr"`
	Kopf    XJustizKopf
	Fach    XJustizFach
}

type XJustizKopf struct {
	Zeitpunkt string `xml:"erstellungszeitpunkt"`
	Absender  string `xml:"absender"`
}

type XJustizFach struct {
	Token string `xml:"token"`
}

func GenerateXJustiz(token string) string {
	msg := XJustizNachricht{
		Xmlns: "http://www.xjustiz.de",
		Kopf: XJustizKopf{
			Zeitpunkt: time.Now().Format("2006-01-02T15:04:05"),
			Absender:  "L.S.S. Wusulum Core (Go)",
		},
		Fach: XJustizFach{Token: token},
	}
	out, err := xml.MarshalIndent(msg, "", "  ")
	if err != nil {
		return fmt.Sprintf("<error>%s</error>", err.Error())
	}
	return string(out)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. APEX CONTROLLER
// ═══════════════════════════════════════════════════════════════════════════════

func ProcessApex(req ApexRequest) ApexResponse {
	// 1. Forensik
	isHostile, hits := ScanSignatures(req.Message)
	threat := 0.1
	if isHostile {
		threat = 0.8
	} else if req.IsThreat {
		threat = 0.6
	}

	// 2. Biologie
	bioState := polyvagal.Update(threat)

	// 3. Kortex
	kortexMode := "CREATIVE_MODE"
	if isHostile || strings.Contains(bioState, "SHUTDOWN") {
		kortexMode = "DEFENSIVE_MODE"
	}

	// 4. Advokatus
	if isHostile {
		inmate := gotham.Incarcerate(req.Message, fmt.Sprintf("Signatures: %v", hits))
		polyvagal.CoRegulate(0.2)
		return ApexResponse{
			Status:        "CONTAINED",
			BioState:      bioState,
			ThreatLevel:   threat,
			SignatureHits: hits,
			GothamInmate:  inmate,
		}
	}

	// 5. Wusulum
	wusCycle := wusulum.Cycle()

	// 6. Foucault
	foucaultResult := FoucaultInvert(req.Message)

	// 7. XJustiz
	xj := GenerateXJustiz(req.Message)

	// 8. Seal
	block := ledger.Append("APEX_PROCESSED", xj)

	return ApexResponse{
		Status:         "PROCESSED",
		BioState:       bioState,
		KortexMode:     kortexMode,
		ThreatLevel:    threat,
		Wusulum:        wusCycle,
		FoucaultResult: foucaultResult,
		XJustizOutput:  xj,
		AsmeSeal:       block.Hash,
	}
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. MIDDLEWARE (CORS + Logging)
// ═══════════════════════════════════════════════════════════════════════════════

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", cfg.AllowedOrigin)
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func loggingMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		next.ServeHTTP(w, r)
		log.Printf("[%s] %s %s (%v)", r.Method, r.URL.Path, r.RemoteAddr, time.Since(start))
	})
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. HANDLERS
// ═══════════════════════════════════════════════════════════════════════════════

func handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":         "HEALTHY",
		"version":        "26.1.0-go",
		"uptime_seconds": int(time.Since(ledger.started).Seconds()),
		"worm_block":     ledger.Count(),
		"worm_last_hash": ledger.LastHash()[:32] + "...",
		"features": map[string]bool{
			"wusulum":    true,
			"polyvagal":  true,
			"xjustiz":    true,
			"gotham":     true,
			"websocket":  true,
		},
	})
}

func handleStats(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"worm": map[string]interface{}{
			"total_blocks": ledger.Count(),
			"ring_size":    len(ledger.blocks),
			"last_hash":    ledger.LastHash()[:32] + "...",
		},
		"lss": map[string]interface{}{
			"entropy":        wusulum.Entropy(),
			"bio_state":      polyvagal.State(),
			"brummel_phase":  brummel.Phase(),
			"gotham_inmates": gotham.Count(),
		},
		"uptime_seconds": int(time.Since(ledger.started).Seconds()),
	})
}

func handleLedgerInspect(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	blocks := ledger.Tail(20)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"count":    ledger.Count(),
		"blocks":   blocks,
		"returned": len(blocks),
	})
}

func handleGotham(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"inmates": gotham.List(),
		"count":   gotham.Count(),
	})
}

func handleApex(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	var req ApexRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if req.AuthSeal != cfg.ExpectedSeal {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusForbidden)
		json.NewEncoder(w).Encode(map[string]string{
			"status": "403_FORBIDDEN", "detail": "Invalid Seal",
		})
		return
	}
	resp := ProcessApex(req)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

func handleBrummel(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	var req BrummelRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if req.AuthSeal != cfg.ExpectedSeal {
		w.WriteHeader(http.StatusForbidden)
		json.NewEncoder(w).Encode(map[string]string{"status": "403_FORBIDDEN"})
		return
	}
	osc := brummel.Oscillate()
	h := sha256.Sum256([]byte(fmt.Sprintf("DUMMELT_%s_%d", req.Message, osc)))
	result := map[string]interface{}{
		"status":             "CONTAINED",
		"entity":             "[0003]",
		"universe":           "Brummel_Grummel_brummelt&_Dummelt",
		"oscillation_state":  osc,
		"lineage_doctrine":   "45[LINEAGE] 65[DOCTRINE]",
		"dummelt_hash":       hex.EncodeToString(h[:])[:16],
	}
	ledger.Append("BRUMMEL_INJECTION_CONTAINED", req.Message)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(result)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. WEBSOCKET TELEMETRIE
// ═══════════════════════════════════════════════════════════════════════════════

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

func handleWebSocket(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Print("upgrade failed: ", err)
		return
	}
	defer conn.Close()
	log.Println("[WS] Client verbunden")

	for {
		data := map[string]interface{}{
			"ts":             time.Now().Unix(),
			"throughput":     250000 + rand.Intn(40000),
			"latency":        8.5 + rand.Float64()*2.0,
			"entropy":        wusulum.Entropy(),
			"bio_state":      polyvagal.State(),
			"worm_block":     ledger.Count(),
			"gotham_inmates": gotham.Count(),
			"brummel_phase":  brummel.Phase(),
		}
		if err := conn.WriteJSON(data); err != nil {
			log.Println("[WS] Fehler beim Senden:", err)
			return
		}
		time.Sleep(1500 * time.Millisecond)
	}
}

// ═══════════════════════════════════════════════════════════════════════════════
// 12. MIN/MAX HELPER
// ═══════════════════════════════════════════════════════════════════════════════

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func minF(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

func maxF(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}

// ═══════════════════════════════════════════════════════════════════════════════
// 13. MAIN
// ═══════════════════════════════════════════════════════════════════════════════

func main() {
	loadConfig()

	// Initial-Block im Ledger
	ledger.Append("GENESIS", cfg.MasterSeal)

	mux := http.NewServeMux()
	mux.HandleFunc("/health", handleHealth)
	mux.HandleFunc("/stats", handleStats)
	mux.HandleFunc("/ledger/inspect", handleLedgerInspect)
	mux.HandleFunc("/api/v1/gotham", handleGotham)
	mux.HandleFunc("/api/v1/apex/process", handleApex)
	mux.HandleFunc("/api/v1/sandbox/brummel", handleBrummel)
	mux.HandleFunc("/ws/telemetry", handleWebSocket)

	// Root-Redirect
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]string{
			"service": "GO_BLUE_CORE",
			"version": "26.1.0",
			"docs":    "/health, /stats, /ledger/inspect, /api/v1/apex/process",
		})
	})

	handler := loggingMiddleware(corsMiddleware(mux))

	// Sortiere Signaturen für deterministische Ausgabe
	sort.Strings(signatures)

	log.Println("═══════════════════════════════════════════════════════════════")
	log.Printf("🚀 GO BLUE CORE v26.1.0 startet auf Port %s", cfg.Port)
	log.Printf("   Seal:       %s", cfg.ExpectedSeal)
	log.Printf("   Ledger-Cap: %d", cfg.LedgerCap)
	log.Printf("   Signaturen: %d", len(signatures))
	log.Println("═══════════════════════════════════════════════════════════════")

	if err := http.ListenAndServe(":"+cfg.Port, handler); err != nil {
		log.Fatal("Server-Fehler:", err)
	}
}
