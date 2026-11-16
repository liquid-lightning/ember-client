Copied verbatim from [`liquid-lightning/ember`](https://github.com/liquid-lightning/ember)
at commit `21a3868b22970de3926c8d02e2894258fab4f540`:

- `proto/ember/scheduler.proto`

`assets.proto` is copied from the same repo too, at a newer commit (update
this note with its hash once that change is committed there) -
ember-server's own client-facing contract for cross-asset payments (see
that proto file's own header comment).

This is the schema ember-server owns itself: node lifecycle
(register/recover/status/list/delete) and cross-asset payments. Do not
hand-edit `scheduler.proto`/`assets.proto`; re-copy them from ember at a
newer commit and update this note instead.
