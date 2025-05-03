// fn main() {
//     tonic_build::configure()
//         .compile_protos(&["proto/ssh.proto"], &["proto"])
//         .unwrap_or_else(|e| panic!("Failed to compile protos {e:?}"));
// }

fn main() {
    tonic_build::compile_protos("proto/ssh.proto").unwrap();
}
