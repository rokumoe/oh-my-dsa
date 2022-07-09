fn main() {
    println!("cargo:rerun-if-changed=src/proto/paxos.proto");
    tonic_build::configure()
        .out_dir("src/proto")
        .compile(&["src/proto/paxos.proto"], &["src/proto"])
        .expect("compile protos")
}
