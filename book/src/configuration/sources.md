# Sources

Sources represent the "origin" of the events processed by Scrolls. Any compatible source is responsible for feeding blockchain data into crolls's pipeline for further processing. This section describes the currently available sources included as part the main Scrolls codebase.

### Node-to-Node
The Node-to-Node (N2N) source uses Ouroboros mini-protocols to connect to a local or remote Cardano node through a tcp socket bearer and fetches block data using the ChainSync mini-protocol instantiated to "headers only" and the BlockFetch mini-protocol for retrieval of the actual block payload.


### Node-to-Client 
The Node-to-Client (N2C) source uses Ouroboros mini-protocols to connect to a local Cardano node through a unix socket bearer and fetches block data using the ChainSync mini-protocol instantiated to "full blocks".


