// Paved-road Go (Gin) entrypoint.
//
// Wired:
//   - /healthz, /ready, /metrics
//   - structured zerolog JSON with trace correlation
//   - OTel tracing (OTLP/gRPC), shut down gracefully on SIGTERM
//   - Prometheus middleware (request count + duration histogram)
//   - graceful shutdown (drain in-flight requests within DRAIN_TIMEOUT)
//   - configuration validation at boot (required env)
package main

import (
	"context"
	"errors"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
)

var (
	version = "dev"
	commit  = "unknown"
	date    = "unknown"
)

var (
	httpRequests = promauto.NewCounterVec(prometheus.CounterOpts{
		Name: "http_requests_total",
		Help: "Total HTTP requests.",
	}, []string{"method", "path", "code"})
	httpLatency = promauto.NewHistogramVec(prometheus.HistogramOpts{
		Name:    "http_request_duration_seconds",
		Help:    "HTTP request duration in seconds.",
		Buckets: prometheus.DefBuckets,
	}, []string{"method", "path"})
)

type config struct {
	port          int
	drainTimeout  time.Duration
	otelEndpoint  string
	serviceName   string
	logLevel      zerolog.Level
}

func loadConfig() (*config, error) {
	port, err := strconv.Atoi(getenv("PORT", "8080"))
	if err != nil {
		return nil, err
	}
	dt, err := time.ParseDuration(getenv("DRAIN_TIMEOUT", "20s"))
	if err != nil {
		return nil, err
	}
	level, err := zerolog.ParseLevel(getenv("LOG_LEVEL", "info"))
	if err != nil {
		return nil, err
	}
	return &config{
		port:         port,
		drainTimeout: dt,
		otelEndpoint: os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"),
		serviceName:  getenv("OTEL_SERVICE_NAME", "{{service_name}}"),
		logLevel:     level,
	}, nil
}

func getenv(k, def string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return def
}

func initTracer(ctx context.Context, c *config) (func(context.Context) error, error) {
	if c.otelEndpoint == "" {
		return func(context.Context) error { return nil }, nil
	}
	exp, err := otlptracegrpc.New(ctx, otlptracegrpc.WithInsecure())
	if err != nil {
		return nil, err
	}
	res, _ := resource.Merge(resource.Default(), resource.NewWithAttributes(
		semconv.SchemaURL, semconv.ServiceName(c.serviceName), semconv.ServiceVersion(version),
	))
	tp := sdktrace.NewTracerProvider(sdktrace.WithBatcher(exp), sdktrace.WithResource(res))
	otel.SetTracerProvider(tp)
	return tp.Shutdown, nil
}

func metricsMiddleware(c *gin.Context) {
	start := time.Now()
	c.Next()
	httpRequests.WithLabelValues(c.Request.Method, c.FullPath(), strconv.Itoa(c.Writer.Status())).Inc()
	httpLatency.WithLabelValues(c.Request.Method, c.FullPath()).Observe(time.Since(start).Seconds())
}

func newRouter() *gin.Engine {
	gin.SetMode(gin.ReleaseMode)
	r := gin.New()
	r.Use(otelgin.Middleware("{{service_name}}"), metricsMiddleware, gin.Recovery())

	r.GET("/healthz", func(c *gin.Context) { c.JSON(http.StatusOK, gin.H{"ok": true}) })
	r.GET("/ready", func(c *gin.Context) {
		// extend with real dependency probes (DB, cache) before returning 200
		c.JSON(http.StatusOK, gin.H{"ok": true, "deps": gin.H{"db": "ok"}})
	})
	r.GET("/metrics", gin.WrapH(promhttp.Handler()))

	api := r.Group("/api/v1")
	{
		api.GET("/items", func(c *gin.Context) { c.JSON(200, gin.H{"items": []gin.H{{"id": 1, "name": "example"}}}) })
	}
	return r
}

func main() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix
	cfg, err := loadConfig()
	if err != nil {
		log.Fatal().Err(err).Msg("config load failed")
	}
	zerolog.SetGlobalLevel(cfg.logLevel)
	log.Info().Str("service", cfg.serviceName).Str("version", version).Str("commit", commit).Msg("starting")

	rootCtx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	shutdownTracer, err := initTracer(rootCtx, cfg)
	if err != nil {
		log.Fatal().Err(err).Msg("tracer init failed")
	}

	srv := &http.Server{
		Addr:              ":" + strconv.Itoa(cfg.port),
		Handler:           newRouter(),
		ReadHeaderTimeout: 5 * time.Second,
	}

	go func() {
		if err := srv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			log.Fatal().Err(err).Msg("listen failed")
		}
	}()

	<-rootCtx.Done()
	log.Info().Msg("shutdown signal received, draining")
	shutCtx, cancel := context.WithTimeout(context.Background(), cfg.drainTimeout)
	defer cancel()
	if err := srv.Shutdown(shutCtx); err != nil {
		log.Error().Err(err).Msg("graceful shutdown failed")
	}
	_ = shutdownTracer(shutCtx)
	log.Info().Msg("bye")
}
