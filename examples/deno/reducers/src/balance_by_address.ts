import type { JsonValue } from "npm:@bufbuild/protobuf";
import * as UtxoRpc from "npm:@utxorpc-web/cardano-spec";
import { C } from "./lib/mod.ts";

enum Method {
  Apply = "apply",
  Undo = "undo",
}

enum Action {
  Produce = "produce",
  Consume = "consume",
}

function processTxOutput(txOuput: UtxoRpc.TxOutput, addressType: string, action: Action) {
  const address = C.Address.from_bytes(txOuput.address);

  let key: string;

  switch (addressType) {
    case "payment":
      if (address.as_byron()) {
        // @ts-ignore: checked if address.as_byron() is undefined
        key = address.as_byron()?.to_base58();
      } else if (address.to_bech32(undefined)) {
        key = address.to_bech32(undefined);
      } else {
        const addressHex = Array.from(
          txOuput.address,
          (byte) => byte.toString(16).padStart(2, "0"),
        ).join("");
        throw new Error(`address "${addressHex}" could not be parsed!`);
      }
      break
    case "stake":
      if (address.as_base()) {
        const network_id = address.network_id();
        const stake_cred = address.as_base()?.stake_cred();
    
        key = C.RewardAddress
          // @ts-ignore: checked if address.as_base() is undefined
          .new(network_id, stake_cred)
          .to_address()
          .to_bech32(undefined);
      } else {
        return null;
      }
      break
    default:
      throw new Error(`address type "${addressType}" not implemented`);
  }

  let value;
  switch (action) {
    case Action.Consume:
      value = -txOuput.coin;
      break;
    case Action.Produce:
      value = txOuput.coin;
      break;
  }

  return { key, value };
}

function processBlock(
  blockJson: JsonValue,
  config: Record<string, string>,
  method: Method,
) {
  const block = UtxoRpc.Block.fromJson(blockJson);
  const addressType = config.addressType
  const prefix = config.prefix

  const deltas: Record<string, bigint> = {};
  for (const tx of block.body?.tx ?? []) {
    for (const txOutput of tx.outputs) {
      let action: Action;
      switch (method) {
        case Method.Apply:
          action = Action.Produce;
          break;
        case Method.Undo:
          action = Action.Consume;
          break;
      }

      const delta = processTxOutput(txOutput, addressType, action);
      if (delta) {
        if (delta.key in deltas) {
          deltas[delta.key] += delta.value;
        } else {
          deltas[delta.key] = delta.value;
        }
      }
    }

    for (const txInput of tx.inputs) {
      const txOutput = txInput.asOutput;
      if (txOutput) {
        let action: Action;
        switch (method) {
          case Method.Apply:
            action = Action.Consume;
            break;
          case Method.Undo:
            action = Action.Produce;
            break;
        }

        const delta = processTxOutput(txOutput, addressType, action);
        if (delta) {
          if (delta.key in deltas) {
            deltas[delta.key] += delta.value;
          } else {
            deltas[delta.key] = delta.value;
          }
        }
      }
    }
  }

  const commands = [];
  for (const [key, value] of Object.entries(deltas)) {
    commands.push({
      command: "PNCounter",
      key: prefix + "." + key,
      value: value.toString(),
    });
  }

  return commands;
}

export function apply(blockJson: JsonValue, config: Record<string, string>) {
  return processBlock(blockJson, config, Method.Apply);
}

export function undo(blockJson: JsonValue, config: Record<string, string>) {
  return processBlock(blockJson, config, Method.Undo);
}
