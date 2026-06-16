// Paved-road ASP.NET Core 9 minimal API.
// Wired: /healthz, /ready, /metrics, /api/v1/items + Serilog JSON + OTel + Prometheus + graceful shutdown.

using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using Prometheus;
using Serilog;
using Serilog.Formatting.Compact;

var serviceName = Environment.GetEnvironmentVariable("OTEL_SERVICE_NAME") ?? "{{service_name}}";
var port        = int.Parse(Environment.GetEnvironmentVariable("PORT") ?? "8080");

// Structured JSON logging — emits trace_id/span_id when an OTel span is active.
Log.Logger = new LoggerConfiguration()
    .Enrich.FromLogContext()
    .Enrich.WithProperty("service", serviceName)
    .WriteTo.Console(new RenderedCompactJsonFormatter())
    .CreateLogger();

var builder = WebApplication.CreateBuilder(args);
builder.Host.UseSerilog();
builder.WebHost.ConfigureKestrel(o => o.ListenAnyIP(port));

// Graceful-shutdown drain window aligned with Pod terminationGracePeriodSeconds.
builder.Services.Configure<HostOptions>(opt =>
{
    var drain = int.Parse(Environment.GetEnvironmentVariable("DRAIN_TIMEOUT_SECONDS") ?? "25");
    opt.ShutdownTimeout = TimeSpan.FromSeconds(drain);
});

// Health probes — separate liveness vs readiness.
builder.Services.AddHealthChecks()
    .AddCheck("self", () => Microsoft.Extensions.Diagnostics.HealthChecks.HealthCheckResult.Healthy(), tags: new[] { "liveness" });

// OpenTelemetry — OTLP/gRPC when endpoint provided.
builder.Services.AddOpenTelemetry()
    .ConfigureResource(r => r.AddService(serviceName))
    .WithTracing(t =>
    {
        t.AddAspNetCoreInstrumentation();
        if (!string.IsNullOrEmpty(Environment.GetEnvironmentVariable("OTEL_EXPORTER_OTLP_ENDPOINT")))
            t.AddOtlpExporter();
    });

var app = builder.Build();

app.UseRouting();
app.UseHttpMetrics();                                    // Prometheus auto-metrics

app.MapHealthChecks("/healthz", new() { Predicate = h => h.Tags.Contains("liveness") });
app.MapHealthChecks("/ready");
app.MapMetrics("/metrics");                              // Prometheus exposition

app.MapGet("/api/v1/items", () => new { items = new[] { new { id = 1, name = "example" } } });

await app.RunAsync();
