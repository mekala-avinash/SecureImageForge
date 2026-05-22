module github.com/acme/{{service_name}}

go 1.23

require (
	github.com/gin-gonic/gin v1.10.0
	github.com/prometheus/client_golang v1.20.5
	github.com/rs/zerolog v1.33.0
	go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin v0.57.0
	go.opentelemetry.io/otel v1.32.0
	go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.32.0
	go.opentelemetry.io/otel/sdk v1.32.0
	go.opentelemetry.io/otel/trace v1.32.0
)
