# Testdrive

In the `testdrive` folder you'll find a minimal example that uses docker-compose to spin up a local Redis instance and a Scrolls daemon. You'll need Docker and docker-compose installed in your local machine. Run the following commands to start it:

```sh
cd testdrive
docker-compose up
```

You should see the logs of both _Redis_ and _Scrolls_ crawling the chain from a remote relay node. If you're familiar with Redis CLI, you can run the following commands to see the data being cached:

```sh
redis:6379> KEYS *
1) "c1.addr1qx0w02a2ez32tzh2wveu80nyml9hd50yp0udly07u5crl6x57nfgdzya4axrl8mfx450sxpyzskkl95sx5l7hcfw59psvu6ysx"
2) "c1.addr1qx68j552ywp6engr2s9xt7aawgpmr526krzt4mmzc8qe7p8qwjaawywglaawe74mwu726w49e8e0l9mexcwjk4kqm2tq5lmpd8"
3) "c1.addr1q90z7ujdyyct0jhcncrpv5ypzwytd3p7t0wv93anthmzvadjcq6ss65vaupzmy59dxj43lchlra0l482rh0qrw474snsgnq3df"
4) "c1.addr1w8vg4e5xdpad2jt0z775rt0alwmku3my2dmw8ezd884zvtssrq6fg"
5) "c1.addr1q9tj3tdhaxqyph568h7efh6h0f078m2pxyd0xgzq47htwe3vep55nfane06hggrc2gvnpdj4gcf26kzhkd3fs874hzhszja3lh"
6) "c1.addr1w8tqqyccvj7402zns2tea78d42etw520fzvf22zmyasjdtsv3e5rz"
redis:6379> SMEMBERS c1.addr1w8tqqyccvj7402zns2tea78d42etw520fzvf22zmyasjdtsv3e5rz
1) "2548228522837ea580bc55a3e6a09479deca499b5e7f3c08602a1f3191a178e7:20"
2) "04086c503512833c7a0c11fc85f7d0f0422db9d14b31275b3d4327c40c6fd73b:25"
redis:6379>
```

Once you're done with the testdive, you can clean your environment by running:

```sh
docker-compose down
```

