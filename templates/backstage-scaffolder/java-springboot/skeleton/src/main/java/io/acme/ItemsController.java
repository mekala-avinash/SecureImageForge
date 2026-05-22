package io.acme.{{service_name}};

import io.micrometer.core.instrument.MeterRegistry;
import io.micrometer.core.instrument.Counter;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.Map;

@RestController
@RequestMapping("/api/v1")
public class ItemsController {

    private final Counter itemsListed;

    public ItemsController(MeterRegistry registry) {
        this.itemsListed = Counter.builder("app_items_listed_total")
                .description("Total times /api/v1/items was called.")
                .register(registry);
    }

    @GetMapping("/items")
    public Map<String, Object> list() {
        itemsListed.increment();
        return Map.of("items", List.of(Map.of("id", 1, "name", "example")));
    }
}
