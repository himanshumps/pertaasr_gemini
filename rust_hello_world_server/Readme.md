## Sample rust server against which we are going to run the benchmark.

This is a very simple hellow world server that we will use to test out HTTP benchmarking client and compare the performance with other benchmarking tools like wrk and plow.

To build the rust server docker image, run this command

```bash
docker build --platform=linux/amd64 --progress=plain -t rust-hello-rest -f Dockerfile.rust_server .
```

This is the command to run the server in case you want to see if it is working.

```bash
docker run --platform=linux/amd64 -d --name=rust-server rust-hello-rest
```

To run the test:

### plow golang:

Run test:

```bash

docker network create plow-network
docker run --platform=linux/amd64 --memory="4g" --cpus="2" -d --name rust-server-for-plow --network plow-network rust-hello-rest
sleep 5
printf "\n\nRunning plow test\n\n"
docker run --platform=linux/amd64  --memory="4g" --cpus="2" --rm --name plow_client --network plow-network ghcr.io/six-ddc/plow http://rust-server-for-plow:8080/ -c20 -d2m --interval=0 --summary

```

Clean up network and containers after the test

```bash

docker ps -a --filter "network=plow-network"
docker stop $(docker ps -a -q --filter "network=plow-network")
docker rm $(docker ps -a -q --filter "network=plow-network")
docker network rm plow-network

```

Output:
```
Benchmarking http://rust-server-for-plow:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888

Summary:
  Elapsed        2m0s
  Count      22182180
    2xx      22182180
  RPS      184849.834
  Reads    22.741MB/s
  Writes   12.164MB/s

Statistics     Min       Mean     StdDev     Max  
  Latency     15µs       107µs     82µs    38.61ms
  RPS       140240.46  184851.66  6407.91  194448 

Latency Percentile:
  P50    P75    P90    P95    P99   P99.9  P99.99 
  91µs  133µs  182µs  212µs  321µs  588µs  1.826ms

Latency Histogram:
  104µs    21513960  96.99%
  171µs      474687   2.14%
  238µs      154942   0.70%
  361µs       27775   0.13%
  533µs        9683   0.04%
  997µs        1100   0.00%
  2.089ms        32   0.00%
  2.234ms         1   0.00%
```

### wrk 

First build the wrk binary for debian as alpine one is giving some issue on my system.

It does take some time, so please be patient.

```bash
docker build --platform=linux/amd64 --progress=plain -t wrk_binary -f Dockerfile.wrk .
```

Run test:

```bash

docker network create wrk-network
docker run --platform=linux/amd64 --memory="4g" --cpus="2" -d --name rust-server-for-wrk --network wrk-network rust-hello-rest
sleep 5
printf "\n\nRunning wrk test\n\n"
docker run --platform=linux/amd64  --memory="4g" --cpus="2" --rm --name wrk_client --network wrk-network wrk_binary -c20 -d2m http://rust-server-for-wrk:8080/

```

Clean up network and containers after the test

```bash

docker ps -a --filter "network=wrk-network"
docker stop $(docker ps -a -q --filter "network=wrk-network")
docker rm $(docker ps -a -q --filter "network=wrk-network")
docker network rm wrk-network

```

Output:

```
Running 2m test @ http://rust-server-for-wrk:8080/
  2 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    90.62us  505.56us  75.00ms   99.48%
    Req/Sec   121.78k    14.15k  147.07k    80.88%
  29073267 requests in 2.00m, 3.49GB read
  Socket errors: connect 0, read 0, write 0, timeout 17
Requests/sec: 242173.20
Transfer/sec:     29.79MB
```


