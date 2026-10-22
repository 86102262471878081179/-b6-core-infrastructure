FROM golang:1.22-alpine
WORKDIR /app

# Proxy, um Netzwerkfehler beim Download zu vermeiden
ENV GOPROXY=https://goproxy.cn,direct

COPY go.mod ./
# Dieser Befehl erstellt automatisch die fehlende go.sum-Datei
RUN go mod tidy
COPY . .

# Baut das Paket im aktuellen Verzeichnis
RUN go build -o /grok-exporter .

EXPOSE 8081
CMD [ "/grok-exporter" ]
