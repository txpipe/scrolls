# What
This is a reference file on how to deploy scrolls and redis into a k8s cluster. Use it as template only.
For this example a new namespace will be created named `txpipe-namespace`

# Before start

* Kubernetes cluster up and running with access to it.
* Ip address of the master node. For this example will be `192.168.0.100`.
* `redis-cli` installed

# Roll up 

It's as easy as follows:
```bash
$ kubectl apply -f scrolls.yaml
```
The following will be shown up:
```bash
namespace/txpipe-namespace created
configmap/txpipe-configmap-volume-scrolls created
deployment.apps/txpipe-deployment-scrolls created
service/txpipe-service-scrolls created
```
And check it using:
```bash
$ kubectl get all -n txpipe-namespace
```
# Access the Redis db

Execute the following command, and remember, the IP address matches my environment. In your environment will be another.

```bash
$ REDIS_IP=192.168.0.100
$ redis-cli -h ${REDIS_IP} -p 30379
```
ANd then if you look for the KEYS
```redis
192.168.0.100:30379> KEYS *
```
A long long result similar to the following will tail:
```
...
44142) "c2.b6b63bc6b3b85a8b6a897763dec85a9e684423f2a2571a773e044c91097a9606"
44143) "c2.3802732284c1a295f450f7575d4cb764d29741016f75f043e6e276dfe83bbe05"
44144) "c2.affc74bdec92100bcb66759cccf4a228c0a0638a9ed29f774c56b6d6f2bfdd0c"
44145) "c2.f864fe44c72aa450243ff1a6f5abc8403971a9470b225172a9751a4d330f91c6"
...
```