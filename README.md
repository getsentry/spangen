# Spangen

A load generator for spans in traces that adheres to the `snuba-spans` schema.

## Usage

```
Options:
  --count           the number of spans to generate in total. spangen will stop
                    generating new traces after this number has been reached,
                    but it will finish started traces and segments. The actual
                    number of spans generated may therefore be higher than this
                    option.
  --throughput      the throughput of spans per second (defaults to no
                    throttling).
  --spans-per-segment
                    the average number of spans per segment (randomized).
  --spans-per-segment-stddev
                    the standard deviation for randomizing the number of spans
                    per segment.
  --segments-per-trace
                    the average number of segments per trace (randomized).
  --segments-per-trace-stddev
                    the standard deviation for randomizing the number of
                    segments per trace.
  --order           the order in which spans are written in a segment.
  --batch-size      the maximum number of spans that will be generated in a
                    single run. This is used to simulate a stream of spans that
                    are generated in batches. SDKs do not typically generate
                    spans one by one, but rather in batches. This parameter
                    controls the maximum number of spans that will be generated
                    in a single run.
  --batch-delay-ms  the delay in milliseconds between consecutive batches of a
                    segment. This is used to simulate an operating SDK that
                    collects spans over time and flushes them in batches with a
                    given delay.
  --batch-delay-stddev
                    the standard deviation for the batch delay in milliseconds.
  --concurrent-traces
                    the number of concurrent traces that interleave on the
                    stream.
  --payload-size    the size of the payload in bytes.
  --tree-depth      the depth of the span tree within each segment.
  --segments-without-root
                    the percentage of segments without an explicit root span
                    (0..100)
  --orgs            the number of organizations.
  --projects        the number of projects per organization.
  --help, help      display usage information
```

## Local Tests

To use spangen locally, use `cargo` to build and run the binary in release mode.
To produce to Kafka, install [kcat](https://github.com/edenhill/kcat) and pipe
the output of spangen there:

```sh
cargo run --release -- --count 10 | kcat -P -b 127.0.0.1:9092 -t topic_name
```

## Docker

We provide a docker image that bundles `spangen` with `kafkacat` to produce the
generated output to a configurable Kafka topic. The image is available at
`ghcr.io/getsentry/spangen`. The following environment variables configure the
connection:

- `KAFKA_BROKER`: The host and port of the broker. Defaults to `kafka-001:9092`.
- `KAFKA_TOPIC`: The name of the topic to produce to. Defaults to `snuba-spans`.
- `KAFKA_PARTITION`: An optional partition number to produce to. Defaults to
  random partitioning.

Example:

```sh
docker run --rm -it -e KAFKA_BROKER=127.0.0.1:9092 ghcr.io/getsentry/spangen --count 10
```
