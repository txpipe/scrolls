# Scrolls testing framework

This is a testing framework for evaluating the output of reducers in scrolls. It uses queries against a cardano db-sync instance to create a ground truth data set for each reducer.

## How to run

1. Launch redis

Bring up a redis instance using `docker compose up -d`

2. Run scrolls

Run scrolls for a small number of blocks by setting the [intersect] and [finalize] parameters in daemon.toml. The more blocks you run for, the slower the tests will be since queries against db-sync will execute much slower. We recommend ~50 blocks or less.

You can run scrolls using `./run.sh`

3. Run tests

Inside tests.rs you need to set up the parameters for your tests. This includes adding your redis and postgres connection strings as well as providing the start and end block hashes which should match those provided in the daemon.toml file.

Finally you can add or remove tests from the tests vector in the main fn.

To perform the tests, simply run `cargo run`

## Future development

- Integrate better with the existing repo
- Finish writing tests for the remaining reducers
- Create a config file for setting up all test parameters