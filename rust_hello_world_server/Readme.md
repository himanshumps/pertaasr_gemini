## Sample rust server against which we are going to run the benchmark.

This is a very simple hellow world server that we will use to test out HTTP benchmarking client and compare the performance with other benchmarking tools like wrk and plow.

To build the docker image, run this command

```bash
docker build --platform=linux/amd64 --progress=plain -t rust-hello-rest .
```

To run the server

```bash
docker run --platform=linux/amd64 -d -p 8085:8080 --name=rust-server rust-hello-rest
```