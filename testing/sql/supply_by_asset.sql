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

, ma_tx_mint_local as (
  select  tx_id
      ,   ident as token_id
      ,   quantity
  from tx_local
  inner join ma_tx_mint on ma_tx_mint.tx_id = tx_local.id
)

select  'supply_by_asset.' || encode(policy, 'hex') || encode(name, 'hex') as asset_id
		, 	cast(supply as varchar) as supply
from (
  select token_id, sum(quantity) as supply
  from ma_tx_mint_local
  group by token_id
) as ma_supply
left join multi_asset on multi_asset.id = ma_supply.token_id