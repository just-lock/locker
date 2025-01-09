# Locker

A simple locker component.

## Deterministic build

```bash
https://github.com/just-lock/locker.git
cd locker
docker pull radixdlt/scrypto-builder:v1.2.0
docker run -v .:/src radixdlt/scrypto-builder:v1.2.0
```

## Verifing onledger

Get compiled code hex:

```bash
xxd -p target/wasm32-unknown-unknown/release/locker.wasm | tr -d '\n' > locker.hex
```

Get onledger code hex:

```bash
curl -X POST 'https://mainnet.radixdlt.com/state/package/page/codes' -H 'Content-Type: application/json' -d '{"package_address": "package_rdx1p4k2vlr6rejahqfdazv2qff7dl5d88dxkpechapfx77exgv96wu8mk"}' | jq -r '.items[0].code_hex' > locker_onledger.hex
```

When deploying to the radix ledger some wasm is removed which results in these files not being identical, however we can see locker_onledger.hex is a subset of locker.hex using:

```bash
grep -F -f locker_onledger.hex locker.hex > /dev/null && echo "Valid" || echo "Not valid"
```
