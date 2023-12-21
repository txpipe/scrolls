import * as reducers from './reducers/dist/mod.js'

const config = [
  {
    name: "BalanceByAddress",
    config: {
      addressType: "payment",
      prefix: "balance_by_address",
    }
  },
  {
    name: "BalanceByAddress",
    config: {
      addressType: "stake",
      prefix: "balance_by_stake_address",
    }
  },
]

export async function reduce(blockJson) {
  return reducers.apply(blockJson, config)
}
