#### Run 1

```
Benchmarking http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888

Summary:
  Elapsed        2m0s
  Count      11474300
    2xx      11474300
  RPS       95619.107
  Reads    11.763MB/s
  Writes    8.754MB/s

Statistics    Min       Mean    StdDev     Max   
  Latency     23µs     208µs     141µs   22.911ms
  RPS       64300.27  95656.15  5570.47  101430.5

Latency Percentile:
  P50     P75    P90    P95    P99   P99.9  P99.99 
  192µs  262µs  360µs  429µs  566µs  806µs  4.047ms

Latency Histogram:
  205µs    11167153  97.32%
  293µs      220666   1.92%
  401µs       62672   0.55%
  507µs       19002   0.17%
  616µs        4232   0.04%
  749µs         513   0.00%
  1.085ms        37   0.00%
  3.226ms        25   0.00%
```

#### Run 2:

```
Benchmarking http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888

Summary:
  Elapsed        2m0s
  Count      11594049
    2xx      11594049
  RPS       96617.050
  Reads    11.886MB/s
  Writes    8.846MB/s

Statistics    Min      Mean    StdDev      Max   
  Latency    24µs     206µs     120µs   14.963ms 
  RPS       81598.6  96625.88  3071.05  101597.14

Latency Percentile:
  P50     P75    P90    P95    P99   P99.9  P99.99 
  190µs  261µs  358µs  426µs  551µs  718µs  1.239ms

Latency Histogram:
  202µs    11391314  98.25%
  393µs      185797   1.60%
  578µs       16575   0.14%
  845µs         330   0.00%
  1.784ms        23   0.00%
  2.83ms          2   0.00%
  3.174ms         4   0.00%
  3.43ms          4   0.00%
```

##### Run 3:

```
Benchmarking http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888

Summary:
  Elapsed        2m0s
  Count      11169480
    2xx      11169480
  RPS       93078.951
  Reads    11.451MB/s
  Writes    8.522MB/s

Statistics    Min       Mean    StdDev      Max   
  Latency     23µs     214µs     219µs   16.626ms 
  RPS       74605.92  93040.39  5855.69  100692.14

Latency Percentile:
  P50     P75    P90    P95    P99    P99.9   P99.99 
  192µs  264µs  365µs  437µs  588µs  3.563ms  8.984ms

Latency Histogram:
  208µs     10733589  96.10%
  339µs       368242   3.30%
  420µs        42536   0.38%
  516µs        19920   0.18%
  642µs         4869   0.04%
  828µs          298   0.00%
  4.572ms         23   0.00%
  11.497ms         3   0.00%
```

#### Run 4:

```
Benchmarking http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888

Summary:
  Elapsed        2m0s
  Count      11687565
    2xx      11687565
  RPS       97396.345
  Reads    11.982MB/s
  Writes    8.917MB/s

Statistics    Min      Mean    StdDev     Max   
  Latency    24µs     204µs    118µs   12.981ms 
  RPS       77994.5  97405.99  2704.2  102415.64

Latency Percentile:
  P50     P75    P90    P95    P99   P99.9  P99.99
  190µs  260µs  356µs  421µs  545µs  704µs  968µs 

Latency Histogram:
  199µs  11245803  96.22%
  284µs    300663   2.57%
  386µs    100247   0.86%
  485µs     30589   0.26%
  574µs      8218   0.07%
  647µs      1535   0.01%
  751µs       507   0.00%
  852µs         3   0.00%
```

#### Run 5;

```
Benchmarking http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/ for 2m0s using 20 connection(s).
@ Real-time charts is listening on http://[::]:18888

Summary:
  Elapsed        2m0s
  Count      11229831
    2xx      11229831
  RPS       93581.895
  Reads    11.513MB/s
  Writes    8.568MB/s

Statistics    Min      Mean    StdDev     Max   
  Latency    22µs     213µs     216µs   26.583ms
  RPS       78746.8  93572.19  5975.85  102846.6

Latency Percentile:
  P50     P75    P90    P95    P99    P99.9   P99.99 
  191µs  263µs  362µs  434µs  580µs  3.174ms  8.948ms

Latency Histogram:
  203µs     10531911  93.79%
  317µs       557835   4.97%
  448µs       118195   1.05%
  633µs        20912   0.19%
  2.294ms        864   0.01%
  6.168ms         99   0.00%
  12.179ms         8   0.00%
  15.419ms         7   0.00%
```