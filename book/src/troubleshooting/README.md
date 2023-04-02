# Troubleshooting

## Error: Missing utxo

```
STAGE: reducers, WORK, PANIC: missing utxo: <hash>#<index>
```

When processing a particular transaction, some reducers could need inputs to be cached in a local database, this is called [enrichment](../configuration/enrich.md). To avoid facing this error, you may need to run Scrolls from origin or some old [intersection point](../configuration/intersect.md), to be sure you have every needed tx input cached in the local db.

Alternatively, you may want to skip this error with:

```
 [policy]
missing_data = "Skip"
```

## Error: chain-sync intersect not found

```
STAGE: n2n, BOOTSTRAP, PANIC: chain-sync intersect not found
```


Stored `_cursor` value doesn't point to a valid pair of `"absolute_slot,block_hash"`
