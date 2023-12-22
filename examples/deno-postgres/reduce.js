export async function reduce(block) {
  let sqlCommands = [
    `
    INSERT INTO
      balance_by_address (address, balance)
    VALUES
      (
        (
          SELECT
            STRING_AGG(
              SUBSTRING(
                'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
                FROM
                  (FLOOR(RANDOM() * 36):: INT + 1) FOR 1
              ),
              ''
            )
          FROM
            generate_series(1, 10)
        ),
        FLOOR(RANDOM() * 10001):: BIGINT
      );
    `,
    `
    INSERT INTO
      balance_by_stake_address (address, balance)
    VALUES
      (
        (
          SELECT
            STRING_AGG(
              SUBSTRING(
                'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
                FROM
                  (FLOOR(RANDOM() * 36):: INT + 1) FOR 1
              ),
              ''
            )
          FROM
            generate_series(1, 10)
        ),
        FLOOR(RANDOM() * 10001):: BIGINT
      );
    `,
  ];

  return sqlCommands;
}
