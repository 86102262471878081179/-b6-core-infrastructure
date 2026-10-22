FROM golang:1.22-alpine
WORKDIR /app

# Proxy, um Netzwerkfehler beim Download zu vermeiden
ENV GOPROXY=https://goproxy.cn,direct

COPY go.mod ./
RUN go mod download
COPY . .

# Baut das Paket im aktuellen Verzeichnis (findet automatisch main.go)
RUN go build -o /grok-exporter .

EXPOSE 8081
CMD [ "/grok-exporter" ]
