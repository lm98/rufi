# Hello RuFi
This project shows how to setup a RuFi program.

## Running the examples
In order to run any example, you'll need to open a terminal inside the project root folder `rufi/examples` and follow the instructions below:

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
