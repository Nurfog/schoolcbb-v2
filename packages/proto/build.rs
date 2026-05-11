fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../protos/academic.proto")?;
    tonic_build::compile_protos("../../protos/reporting.proto")?;
    tonic_build::compile_protos("../../protos/workflow.proto")?;
    Ok(())
}
