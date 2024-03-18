# Building a Python Reducer

This guide explains how to use Python to build a custom reducer for Scrolls. 

## How it works

To build our custom reducer we'll leverage an [Oura](https://github.com/txpipe/oura) pipeline that sync data from a Cardano node, filters the data through a Rust-based Python interpreter plugin and finally persists the records in an Sqlite db.

## Requirements

- Scrolls SDK
- [RustPython](https://github.com/RustPython/RustPython)
- sqlite
- make
- [Oura](https://github.com/txpipe/oura)

## Procedure

### 1. Create the project scaffold

There's some boilerplate code required to setup our reducer. The _Scrolls SDK_ cli provides a command to automatically generate the basic file structure that can be later customized to specific needs.

Run the following command in your shell to scaffold a new Python reducer.

```sh
scrolls-sdk scrolls-sdk scaffold --template reducer-oura-python {NAME}
```

where:
- `{NAME}` is the name of your reducer

### 2. Customize your DB schema

Your custom reducer will require a custom DB schema that reflects your particular requirements. In this scenario we're using Sqlite as our relational data persistence mechanism.

Edit the `init.sql` file inside your reducer code to define your schema using SQL:

```sql
CREATE TABLE my_reducer (
    slot INTEGER NOT NULL,
    {{custom fields}}
);

CREATE INDEX idx_my_reducer_slot ON my_reducer(slot);
```

where:
- `{{custom fields}}` is the definition of the custom fields required by your reducer

### 3. Edit your Python reducer logic

The core of your reducer is the business logic you use to map blocks & transactions from the chain into relevant data for your use case. In this scenario we're using Python code that will be interpreted using _RustPython_.

Edit the `main.py` file inside your reducer code define your business logic:

```python
def map_u5c_tx(tx):
	# the Tx param holds the data of the tx to map
	
  
	# Here is where you get to do something interesting
  # with the data. In this example, we just extract the
  # fee data from the Tx:
	# fee = tx["fee"]

	# As default, we just resend the exact same payload
	output := param

	# Return a new Dict that holds the data that will
  # continue down through the Oura pipeline
	return output
}
```

### 4. Edit the _Oura_ config file

The Cardano node, the WASM plugin and the Sqlite DB is connected together via an _Oura_ pipeline. This pipeline will ensure that data goes through each the required steps (stages) in performant and resilient way.

Edit the `oura.toml` in your custom reducer folder to configure your pipeline:

```toml
[source]
type = "N2N"
peers = ["relays-new.cardano-mainnet.iohk.io:3001"]

[[filters]]
type = "SplitBlock"

[[filters]]
type = "ParseCbor"

[[filters]]
type = "PythonPlugin"
path = "main.py"

[sink]
type = "SqlDb"
connection = "sqilte:./scrolls.db"
apply_template = "INSERT INTO {{reducer}} (slot, {{custom fields}}) VALUES ('\{{point.slot}}', '{{custom values}}');"
undo_template = "DELETE FROM {{reducer}} WHERE slot = \{{point.slot}}"
reset_template = "DELETE FROM {{reducer}} WHERE slot > \{{point.slot}}"
```

where:
- `{{custom fields}}` is the list of custom fields in your schema
- `{{custom values}}` is the expressions to access the custom values in the object returned by your custom logic


### 5. Run your pipeline

Now that everything has been configured, you can start your indexing pipeline. This requires that that the sqlite db is created with the corresponding schema. With all requirements in place, the Oura process can be started.

The `Makefile` provided in your custom reducer files provides a shortcut to trigger the pipeline making sure that all requirements are in place.

Run the following command:

```sh
make run
```
