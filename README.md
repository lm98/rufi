# Rust Fields
Rust Fields is a Rust-based implementation of the Aggregate Computing paradigm, that lets you program the behaviour
of a Collective Adaptive System by manipulating Computational Fields.

This framework is composed of the following modules:
- [RuFi Core](crates/rf-core/README.md): provides the basic concepts and functionalities for the RuFi framework.
- [RuFi Distributed](crates/rf-distributed/README.md): provides types and functionalities for executing RuFi programs in a distributed fashion.

## Structure of this repository
This repository is structured as follows:
- `crates`: contains the library crates of the RuFi framework.
- `examples`: contains some binary applications with examples of RuFi usage.

## Running the tests
In order to run the tests, you'll need to open a terminal inside the project root folder and follow the instructions below:

````shell
cargo test
````

In order to run only the tests of a specific crate, you can use the following command:

````shell
`cargo test -p <crate_name>`
````

for example:

````shell
cargo test -p rf-core
````
## Running the examples
In order to run any example, you'll need to open a terminal inside the project root folder and follow the instructions below:

- Local gradient:
  This example will execute a gradient aggregate algorithm inside a single process.
  The node topology is the following: [1] - [2] - [3] - [4] - [5].
  In order to launch the program and see the output, run:
````shell
docker-compose -f docker-compose.local.yml up
````
- Distributed gradient:
  This example will launch 5 different processes, each one representing a node in the topology and communicating with the others.
  The node topology is the following: [1] - [2] - [3] - [4] - [5].
  In order to launch the program and see the output, run:
````shell
docker-compose -f docker-compose.distributed.yml up
````
N.B It may happen that some nodes take longer to output the correct value.