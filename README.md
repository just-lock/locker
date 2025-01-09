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

When deploying to the radix ledger some wasm is changed. Therefore the only way I am aware of to verify the onledger code is to redeploy the newly comipled code and compare the code hashs using:

```bash
curl -X POST 'https://mainnet.radixdlt.com/state/package/page/codes' -H 'Content-Type: application/json' -d '{"package_address": "package_rdx1p4k2vlr6rejahqfdazv2qff7dl5d88dxkpechapfx77exgv96wu8mk"}'
```

and 

```bash
curl -X POST 'https://mainnet.radixdlt.com/state/package/page/codes' -H 'Content-Type: application/json' -d '{"package_address": "{NEW_PACKAGE}"}'
```
