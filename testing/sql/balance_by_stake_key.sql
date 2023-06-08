with
tx_local as (
  select id
  from tx
  where block_id > (
    select id
    from block
    where hash = '\x{{ start_hash }}'
  ) and block_id <= (
    select id
    from block
    where hash = '\x{{ end_hash }}'
  )
)

, tx_out_local as (
  select  tx_id
      ,   index
      ,   address
      ,   stake_address_id
      ,   value
  from tx_local 
  inner join tx_out on tx_id = tx_local.id
)

, tx_in_local as (
  select  tx_in_id as tx_id
      ,   null::int as index
      ,   address
      ,   stake_address_id
      ,   -value as value
  from tx_local
  inner join tx_in on tx_in_id = tx_local.id 
  inner join tx_out_local on tx_out_local.tx_id = tx_out_id and tx_out_local.index = tx_out_index
)

select  'balance_by_stake_key.' || stake_address.view as stake_key
		, 	cast(value as varchar) as balance
from (
  select  stake_address_id
      , 	sum(value) as value
  from (
    select *
    from tx_out_local
    union all
    select *
    from tx_in_local
  ) as tx_out_and_in
  group by stake_address_id
) as balance_by_stake_address_id
inner join stake_address on stake_address.id = stake_address_id