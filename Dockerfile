FROM golang:1.22-alpine
WORKDIR /app
COPY go.mod ./
RUN go mod download
COPY . .
RUN go build -o /grok-exporter *.go
EXPOSE 8081
CMD [ "/grok-exporter" ]
