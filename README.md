# Create one of the fastest HTTP Benchmarking Tool

This repo is for trying to create one of the fastest HTTP Benchmarking Client using ☕ and 🦀 using Gemini.

The features that we are looking for:
1. Each request should be customizable (path, headers, query params ...)
2. Should be developers friendly while using Java 11 or higher
3. Uses Project Panama (specifically JEP 454, Foreign Function & Memory API) provides a modern, safe, and efficient alternative to JNI for calling native C/C++ libraries from Java or _allows native libraries to call Java._
4. Should support various metrics like request sent and responses received, bytes sent and received, response time percentiles and ...
5. Graphs to view the reported metrics
6. Log outliers like response code mismatch or response time sla breach
7. Rate Limiter (if time permits)

Everyone is talking about calling Rust from Java as Rust is crazy fast, but Java is not that far behind, see [One Billion Row Challenge](https://github.com/gunnarmorling/1brc).

Creating a tool in rust where it allows every permutation and combination of request is hard. Take e.g. HMAC where the token may be generated using specific url/body/query param. Also, there are lot of application which still uses legacy code which generate metadata for the request which will be time taking to port to Rust.

The idea behind this benchmarking tool is to provide developers use Java, customize the request and provide it to rust for execution.
