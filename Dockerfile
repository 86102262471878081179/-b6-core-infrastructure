FROM golang:1.22-alpine
WORKDIR /app

# Proxy für stabile Downloads
ENV GOPROXY=https://goproxy.cn,direct

# KOPIERE ZUERST ALLES (wichtig für go mod tidy)
COPY . .

# Jetzt erst die Abhängigkeiten auflösen
RUN go mod tidy

# Jetzt kompilieren
RUN go build -o /grok-exporter .

EXPOSE 8081
CMD [ "/grok-exporter" ]
