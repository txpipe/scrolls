# Building a Golang Reducer

This guide explains how to use Golang to build a custom reducer for Scrolls. 

## How it works

To build our custom reducer we'll leverage an [Oura](https://github.com/txpipe/oura) pipeline that sync data from a Cardano node, filters the data through a Wasm plugin and finally persists the records in an Sqlite db.

## Requirements

- Scrolls SDK
- [tinygo](https://tinygo.org/getting-started/install/)
- sqlite
- make
- [Oura](https://github.com/txpipe/oura)

## Procedure

### 1. Create the project scaffold

There's some boilerplate code required to setup our reducer. The _Scrolls SDK_ cli provides a command to automatically generate the basic file structure that can be later customized to specific needs.

Run the following command in your shell to scaffold a new Golang reducer.

```sh
scrolls-sdk scrolls-sdk scaffold --template reducer-oura-golang {NAME}
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

### 3. Edit your Golang reducer logic

The core of your reducer is the business logic you use to map blocks & transactions from the chain into relevant data for your use case. In this scenario we're using Golang code that compiles to WASM.

Edit the `main.go` file inside your reducer code define your business logic:

```go
package main

import (
	"github.com/extism/go-pdk"
)

//export map_u5c_tx
func map_u5c_tx() int32 {
	// unmarshal the U5C Tx data provided by the host
	var param map[string]interface{}
	err := pdk.InputJSON(&param)

	if err != nil {
		pdk.SetError(err)
		return 1
	}

  // you can log info to see it in the debug output of
  // the pipeline:
	//pdk.Log(pdk.LogInfo, fmt.Sprintf("%v", param))

	// Here is where you get to do something interesting
  // with the data. In this example, we just extract the
  // fee data from the Tx:
	// fee := param["fee"].(interface{})

	// As default, we just resend the exact same payload
	output := param

	// Use this method to return the mapped value back to
  // the Oura pipeline.
	err = pdk.OutputJSON(output)

	if err != nil {
		pdk.SetError(err)
		return 1
	}

	// return 0 for a successful operation and 1 for
  // failure.
	return 0
}

// you need to keep the main entry point even if we
// don't use it
func main() {}
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
type = "WasmPlugin"
path = "plugin.wasm"

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


5. Run your pipeline

Now that everything has been configured, you can start your indexing pipeline. This requires that you compile your Golang code into wasm and that the sqlite db is created with the corresponding schema. With all requirements in place, the Oura process can be started.

The `Makefile` provided in your custom reducer files provides a shortcut to trigger the pipeline making sure that all requirements are in place.

Run the following command:

```sh
make run
```
