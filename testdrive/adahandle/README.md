# Swarm Mode Testdrive

This testdrive example shows how to keep track of a lookup table of $handle => address. 

## How does it work?

This example setups a Scrolls instances with a single reducer. This reducer creates a Redis key for each $handle found. The value of the Redis entry provides the address to which the handle points.

> **Warning**
> This example starts from a very advanced point in mainnet to ensure that we'll get data on Redis without having to go through the origin of the chain. If you plan on using this for production, make sure to crawl the whole chain.

## Getting started

Run the following command from within the directory of this example:

```sh
docker-compose up
```
