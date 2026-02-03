# Create one of the fastest HTTP Benchmarking Tool

This repo is for trying to create one of the fastest HTTP Benchmarking Client using ☕ and 🦀 using Gemini.

The features that we are looking for:
1. Each request should be customizable (path, headers, query params ...)
2. Should be developers friendly while using Java 25 (LTS) or higher
3. Should use Project Panama (specifically JEP 454, Foreign Function & Memory API) which provides a modern, safe, and efficient alternative to JNI for calling native C/C++ libraries from Java or _allows native libraries to call Java._
4. Should support various metrics like request sent and responses received, bytes sent and received, response time percentiles and ...
5. Graphs to view the reported metrics
6. Log outliers like response code mismatch or response time sla breach
7. Rate Limiter (if time permits)

Everyone is talking about calling Rust from Java as Rust is crazy fast, but Java is not that far behind, see [One Billion Row Challenge](https://github.com/gunnarmorling/1brc).

Creating a tool in rust where it allows every permutation and combination of every request is hard. Take e.g. HMAC where the token may be generated using specific url/body/query param. Also, there are lot of applications which still uses legacy code which generate metadata for the request which will be time taking to port to Rust.

The idea behind this benchmarking tool is to provide developers use Java, customize the request and provide it to rust for execution. So Java acts as a supplier whereas Rust acts as the executor.

The setup:
1. An actix rust application (basically a hello world application)
2. Docker image with [latest wrk code](https://github.com/wg/wrk.git) on debian compiled as linux/arm64 (the linux/amd64 image is created as well)
3. plow image from ghcr.io/six-ddc/plow
4. pertaas image built using linux/arm64  (the linux/amd64 image is created as well)
Infrastructure Details:
Mac M1 Pro
- Performance (P) cores 7,8 assigned to rust application
- Performance (P) cores 9,10 assigned to run wrk, plow and pertaasr benchmarks

Benchmark results for a duration of 2 min:
Run 1:
| Application | Total requests executed | RPS |
|-------------|-------------------------|-----|
| wrk         |                         |     |
| plow        |                         |     |
| pertaasr    |                         |     |




