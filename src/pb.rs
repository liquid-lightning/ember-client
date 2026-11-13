//! Generated protobuf/gRPC code, compiled by `build.rs` from the vendored
//! `.proto` files in `proto/` (see `proto/cln/VENDORED.md` and
//! `proto/ember/VENDORED.md` for where each one comes from).

pub mod cln {
    tonic::include_proto!("cln");
}

pub mod ember {
    pub mod scheduler {
        pub mod v1 {
            tonic::include_proto!("ember.scheduler.v1");
        }
    }
}
