# Setup

We are running them on openshift developer sandbox. To deploy the application, follow these steps.

Due to the request and limit, we can run only one test at a time using jobs.

#### Deploy the actix server

```bash
# Set the project to himanshumps-1-dev
oc project himanshumps-1-dev
# Deletes the existing deployment if it exists
oc delete deployment rust-server
# Creates a new deployment with the latest image
oc create deployment rust-server --image=ghcr.io/himanshumps/rust-hello-rest:latest 
# Expose the svc which will be used to run the test.
oc expose deployment rust-server --port=8080 --target-port=8080
# Limit the resources to overcome dev limits
oc set resources deployment rust-server --limits cpu=2,memory=4Gi --requests cpu=0.25,memory=4Gi
# Set the replica to 1
oc scale deployment rust-server --replicas=1
# Expose the route in case we want to test it
oc expose svc/rust-server
```

## Run the test

### wrk

```bash
oc project himanshumps-1-dev
oc delete job wrk-job
oc apply -f wrk.yaml
```

Results:

```
Running 2m test @ http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/
2 threads and 20 connections
Thread Stats Avg Stdev Max +/- Stdev
Latency 145.48us 137.62us 12.24ms 99.51%
Req/Sec 63.60k 8.70k 88.20k 68.21%
15186041 requests in 2.00m, 1.82GB read
Requests/sec: 126544.61
Transfer/sec: 15.57MB
```

### plow

```bash
oc project himanshumps-1-dev
oc delete job plow-job
oc apply -f plow.yaml
```

Results:

```
Benchmarking http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888
Summary:
Elapsed 2m0s
Count 11782213
2xx 11782213
RPS 98185.081
Reads 12.079MB/s
Writes 8.989MB/s
Statistics Min Mean StdDev Max
Latency 22µs 203µs 119µs 13.531ms
RPS 81461.26 98170.34 3019.42 103674.93
Latency Percentile:
P50 P75 P90 P95 P99 P99.9 P99.99
188µs 263µs 358µs 423µs 547µs 708µs 910µs
Latency Histogram:
196µs 11218597 95.22%
323µs 481071 4.08%
408µs 56712 0.48%
479µs 15787 0.13%
547µs 6773 0.06%
635µs 2780 0.02%
743µs 446 0.00%
1.587ms 47 0.00%
```

### pertaasr

Tools we are trying to surpass (HTTP 1.1)

| Tool | Total Requests in 2 min | Requests per second |
|------|-------------------------|---------------------|
| wrk  | 15186041                | 126544.61           |
| plow | 11782213                | 98185.081           |


We will be running multiple runs to see the performance. Let's create a table with the github sha and the binary name for the test.

```bash
oc project himanshumps-1-dev
oc delete job pertaasr-job
oc apply -f pertaasr.yaml
```


| Github sha                               | Binary name | Implementation Reference | Total Requests | Requests per second | 
|------------------------------------------|-------------|--------------------------|----------------|---------------------|
| 91106d4e24f4b4c32ef31a90d89b4399d2451bf4 | pertaasr    | Tokio + hyper            | 15564573       | 129700.89           |
| 799d27c3668b28a184f7772d6ed7a13282de76cc | pertaasr    | Tokio + hyper + Jemalloc | 15053091       | 125438.13           |


