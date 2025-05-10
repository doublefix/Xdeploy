// fn main() {
//     tonic_build::configure()
//         .compile_protos(&["proto/ssh.proto"], &["proto"])
//         .unwrap_or_else(|e| panic!("Failed to compile protos {e:?}"));
// }

fn main() {
    tonic_build::compile_protos("proto/api.proto").unwrap();
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["proto/ansible.proto"], &["proto/"])
        .unwrap();

    tonic_build::compile_protos("proto/api.proto").unwrap();
}
