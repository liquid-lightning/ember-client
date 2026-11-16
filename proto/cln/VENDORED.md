Vendored verbatim from https://github.com/ElementsProject/lightning at tag `v26.06.10`:

- `cln-grpc/proto/node.proto`
- `cln-grpc/proto/primitives.proto`

Do not hand-edit these files. To update, re-fetch both files at a newer tag and update this note.

`assetswap.proto` is NOT vendored from upstream - copied verbatim from
[`liquid-lightning/ember`](https://github.com/liquid-lightning/ember)'s
own `proto/cln/assetswap.proto` (see that file's own header comment and
`VENDORED.md`), which is itself a small grpc service ember patches into
the fork's own `cln-grpc` at image-build time. Keep in sync with ember's
copy, not with upstream CLN.
