fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../protos/academic.proto");
    println!("cargo:rerun-if-changed=../../protos/reporting.proto");
    println!("cargo:rerun-if-changed=../../protos/workflow.proto");
    tonic_build::compile_protos("../../protos/academic.proto")?;
    tonic_build::compile_protos("../../protos/reporting.proto")?;
    tonic_build::compile_protos("../../protos/workflow.proto")?;
    Ok(())
}
