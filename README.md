# alpaca-data-relay
`alpaca-data-relay` is an application for streaming data from the [alpaca](https://alpaca.markets/) brokerage to [apache kafka](https://kafka.apache.org/) in a fault-tolerant way. Messages from alpaca are type-checked and only passed on if they conform to the built-in schema.

## Aspirational
Currently, `alpaca-data-relay` emits data to kafka in json. In the future, data will be transmuted into a more efficient binary wire protocol such as protobuf using [serde-transcode](https://serde.rs/transcode.html).
