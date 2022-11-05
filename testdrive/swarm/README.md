# Swarm Mode Testdrive

This testdrive example shows how to leverage docker-compose to run Scrolls in "swarm mode".

## How does it work?

The docker compose file will start several Scrolls instances, each instance as a different container. The config for each pipeline instructs Scrolls to crawl a different segment of the chain (by setting the intersect & finalize sections).

> **Note**
> The segments were constructed by hand. Since the chain-sync mini-protocol requires slot & hash to initiate, there's no _easy_ way to define the segments programatically. Automating this step is out of scope for this example. 

Each Scrolls instance requires a different config file. To avoid duplicating of common values, we use a feature called 'layered config'. There's a `common.toml` file with the values shared across all instances. The instance-specific values are defined in `daemon-x.toml` files, Scrolls will _overlay_ these values on top of the ones defined in `common.toml`.

> **Warning**
> Swarm mode won't work correctly with reducers that require the 'enrich' stage. For the enrich stage to work correctly, it needs to crawl the chain from origin. Swarm mode relies on crawling the chain from different points. Attempting to run both features simultaneously will yield unknown results.

The data from each Scrolls instance is sent to a single, shared Redis instance. The cursor for each pipeline is kept as a different Redis key to avoid collisions. It should be safe to stop / start the whole swarm, each pipeline will attempt to continue from where it was prior to the restart.

## Getting started

Run the following command from within the directory of this example:

```sh
docker-compose up
```
