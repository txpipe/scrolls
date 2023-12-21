import type { JsonValue } from "npm:@bufbuild/protobuf";
import * as BalanceByAddress from './balance_by_address.ts';

const modules = {
  "BalanceByAddress": BalanceByAddress,
}

type Reducer = {
  name: string;
  config: Record<string, string>;
};

function isKeyOfModules(key: string): key is keyof typeof modules {
  return key in modules;
}

export function apply(blockJson: JsonValue, reducers: Reducer[]) {
  return reducers.flatMap(({ name, config }) => {
    if (isKeyOfModules(name)) {
      return modules[name].apply(blockJson, config);
    }
    throw new Error(`Module with name ${name} does not exist.`);
  });
}

export function undo(blockJson: JsonValue, reducers: Reducer[]) {
  return reducers.flatMap(({ name, config }) => {
    if (isKeyOfModules(name)) {
      return modules[name].undo(blockJson, config);
    }
    throw new Error(`Module with name ${name} does not exist.`);
  });
}
