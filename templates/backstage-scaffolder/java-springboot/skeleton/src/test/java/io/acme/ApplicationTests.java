package io.acme.{{service_name}};

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.autoconfigure.web.servlet.AutoConfigureMockMvc;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.web.servlet.MockMvc;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.content;

@SpringBootTest
@AutoConfigureMockMvc
class ApplicationTests {

    @Autowired MockMvc mvc;

    @Test void healthEndpointUp() throws Exception {
        mvc.perform(get("/actuator/health/liveness")).andExpect(status().isOk());
    }

    @Test void readinessEndpointUp() throws Exception {
        mvc.perform(get("/actuator/health/readiness")).andExpect(status().isOk());
    }

    @Test void prometheusEndpointExposed() throws Exception {
        mvc.perform(get("/actuator/prometheus")).andExpect(status().isOk())
            .andExpect(content().string(org.hamcrest.Matchers.containsString("jvm_threads_live_threads")));
    }
}
