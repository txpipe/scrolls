# Python

## Read the data using Python

Assuming you're using Redis as a storage backend (only one available ATM), we recommend using [redis-py](https://github.com/redis/redis-py) package to talk directly to the Redis instance. This is a very simple code snippet to query a the UTXOs by address.

```python
>>> import redis
>>> r = redis.Redis(host='localhost', port=6379, db=0)
>>> r.smembers("c1.addr1w8tqqyccvj7402zns2tea78d42etw520fzvf22zmyasjdtsv3e5rz")
{b'2548228522837ea580bc55a3e6a09479deca499b5e7f3c08602a1f3191a178e7:20', b'04086c503512833c7a0c11fc85f7d0f0422db9d14b31275b3d4327c40c6fd73b:25'}
```

 The Redis operation being used is `smembers` which return the list of members of a set stored under a particular key. In this case, we query by the value `c1.addr1w8tqqyccvj7402zns2tea78d42etw520fzvf22zmyasjdtsv3e5rz`, where `c1` is the key prefix specified in the config for our particular collection and `addr1w8tqqyccvj7402zns2tea78d42etw520fzvf22zmyasjdtsv3e5rz` is the address we're interested in querying. The response from redis is the list of UTXOs (in the format `{tx-hash}:{output-index}`) that are associated with that particular address.

