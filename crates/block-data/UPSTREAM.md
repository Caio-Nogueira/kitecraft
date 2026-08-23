# Pumpkin block-data provenance

The binary tables in `data/` are generated from Pumpkin's `assets/blocks.json`:

- Repository: <https://github.com/Pumpkin-MC/Pumpkin>
- Revision: `beb6947dfc21a1a781523bf207a3c2740f4928f9`
- Pumpkin version: `0.1.0-dev+26.2-26.40`
- Minecraft version: `26.2`
- Upstream license: GPL-3.0

Regenerate from a checkout at that exact revision:

```sh
python3 tools/pumpkin-import/import_block_data.py /path/to/Pumpkin
```

The importer rejects any other revision. `data/manifest.json` records the source
SHA-256 and table counts. The runtime crate intentionally contains no Pumpkin
server, networking, filesystem, async-runtime, cryptography, or RNG dependency.
