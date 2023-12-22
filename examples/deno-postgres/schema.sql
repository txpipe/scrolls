CREATE TABLE balance_by_address (
    address TEXT PRIMARY KEY,
    balance BIGINT NOT NULL
);

CREATE TABLE balance_by_stake_address (
    address TEXT PRIMARY KEY,
    balance BIGINT NOT NULL
);
