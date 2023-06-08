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
      ,   index as tx_index
      ,   address
      ,   ident as token_id
      ,   quantity
  from tx_local
  inner join tx_out on tx_id = tx_local.id
  inner join lateral (
    select tx_out_id, ident, quantity
    from ma_tx_out
    where tx_out_id = tx_out.id
  ) as ma_tx_out on tx_out_id = tx_out.id
)

, tx_in_local as (
  select  tx_in_id as tx_id
      ,   null::int as tx_index
      ,   address
      ,   token_id
      ,   -quantity as quantity
  from tx_local
  inner join tx_in on tx_in_id = tx_local.id 
  inner join tx_out_local on tx_out_local.tx_id = tx_out_id and tx_out_local.tx_index = tx_out_index
)

select  'assets_by_address.' || address as address
		, 	json_agg(json_build_object(
          'key', encode(policy, 'hex') || encode(name, 'hex'),
          'value', cast(quantity as varchar)
        ))::varchar
from (
  select  address
      ,   token_id
      ,   sum(quantity) as quantity
  from (
    select *
    from tx_out_local
    union all
    select *
    from tx_in_local
  ) as tx_out_and_in
  group by address, token_id
) as token_quantity
left join multi_asset on multi_asset.id = token_id
where quantity > 0
group by address