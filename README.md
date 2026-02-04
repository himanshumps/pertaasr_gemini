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

Docker commands ran to test:

```bash
# wrk test
docker network create test-network
docker pull ghcr.io/himanshumps/rust-server-arm64
docker run --platform=linux/arm64  --cpuset-cpus="6,7" --memory="4g" -d --name rust-server --network test-network ghcr.io/himanshumps/rust-server-arm64
sleep 10
docker pull --platform=linux/arm64 ghcr.io/himanshumps/wrk-arm64
printf "\n\n\n\n\nRunning wrk test\n\n"
docker run --platform=linux/arm64  --cpuset-cpus="8,9" --memory="4g" --rm --name wrk_client --network test-network ghcr.io/himanshumps/wrk-arm64 -c40 -d2m http://rust-server:8080/
# plow test
docker stop $(docker ps -a -q --filter "network=test-network")
docker rm $(docker ps -a -q --filter "network=test-network")
docker run --platform=linux/arm64  --cpuset-cpus="6,7" --memory="4g" -d --name rust-server --network test-network ghcr.io/himanshumps/rust-server-arm64
sleep 10
printf "\n\n\n\n\nRunning plow test\n\n"
docker pull --platform=linux/amd64 ghcr.io/six-ddc/plow
docker run --platform=linux/amd64 --cpuset-cpus="8,9" --memory="4g" --rm --name plow_client --network test-network ghcr.io/six-ddc/plow http://rust-server:8080/ -c40 -d2m --interval=0 --summary
# pertaasr test
docker stop $(docker ps -a -q --filter "network=test-network")
docker rm $(docker ps -a -q --filter "network=test-network")
docker run --platform=linux/arm64  --cpuset-cpus="6,7" --memory="4g" -d --name rust-server --network test-network ghcr.io/himanshumps/rust-server-arm64
sleep 10
docker pull --platform=linux/arm64 ghcr.io/himanshumps/pertaasr-arm64
printf "\n\n\n\n\nRunning pertaasr test\n\n"
docker run --platform=linux/arm64 --cpuset-cpus="8,9" --cpu-period=100000 --cpu-quota=-1 --memory="4g" --rm --name pertaasr_client --network test-network ghcr.io/himanshumps/pertaasr-arm64 rust-server:8080 120 40
# Network clean up
docker ps -a --filter "network=test-network"
docker stop $(docker ps -a -q --filter "network=test-network")
docker rm $(docker ps -a -q --filter "network=test-network")
docker network rm test-network
```



Benchmark results for a duration of 2 min

Run 1:

| Application | Total requests  | Requests/sec |
|-------------|-----------------|--------------|
| wrk         | 39169581        | 326344.57    |
| plow        | 25376378        | 211468.152   |
| pertaasr    | 38986240        | 324847.39    |

Percentage difference: 0.47%

Run 2:

| Application | Total requests  | Requests/sec |
|-------------|-----------------|--------------|
| wrk         | 40678785        | 338957.99    |
| plow        | 25081438        | 209010.137   |
| pertaasr    | 38970880        | 324712.30    |

Percentage difference: 4.29%

Run 3:

| Application | Total requests  | Requests/sec |
|-------------|-----------------|--------------|
| wrk         | 40733520        | 339405.44    |
| plow        | 24672274        | 205601.286   |
| pertaasr    | 40087040        | 334013.65    |

Percentage difference: 1.60%


