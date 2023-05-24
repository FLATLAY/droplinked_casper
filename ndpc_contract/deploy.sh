casper-client put-deploy -n http://89.58.52.245:7777 \
--chain-name casper-test --payment-amount 197420060000 \
-k keys/m.pem --session-path deploy/contract.wasm \
--session-arg "ratio_verifier:string='0144f5adf499591351807bc83490314262bd6846beee80a16269a83c9901ecec8a'" \
--session-arg "payment_contract:string='contract-911aae79b634e7aa3d50ca74bafb1bc889f38adf760a154de16c02841efd6bc6'" \
--session-arg "fee:u64='100'" \
--ttl "5hour"