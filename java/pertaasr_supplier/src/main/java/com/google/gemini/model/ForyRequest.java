package com.google.gemini.model;

import java.util.Map;

public record ForyRequest(
    String label,
    String absolute_url,
    String host,
    Integer port,
    String method,
    String path,
    int[] valid_status_codes, // Response predicates can fail a request when the response does not match some criteria.
    Long max_response_time_sla_micros,
    Boolean insecure,
    Map<String, String> headers,
    Map<String, String> query_params,
    String body
){};
