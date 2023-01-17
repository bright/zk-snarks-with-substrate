use node_template_runtime::pallet_zk_snarks::{
	common::{prepare_proof, prepare_verification_key},
	deserialization::{deserialize_public_inputs, Proof, VKey},
	verify::{prepare_public_inputs, verify},
};
use sc_cli::RunCmd;
use std::{fs::File, io::Read};

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Key management cli utilities
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Sub-commands concerned with benchmarking.
	#[command(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Try some command against runtime state.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),

	/// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
	#[cfg(not(feature = "try-runtime"))]
	TryRuntime,

	/// Db meta columns information.
	ChainInfo(sc_cli::ChainInfoCmd),

	ZkSnarksVerify(ZkSnarksVerifyCmd),
}
#[derive(Debug, Clone, clap::Parser)]
pub struct ZkSnarksVerifyCmd {
	#[allow(missing_docs)]
	pub vk_path: String,

	#[allow(missing_docs)]
	pub proof_path: String,

	#[allow(missing_docs)]
	pub inputs_path: String,
}

impl ZkSnarksVerifyCmd {
	pub fn run(&self) -> sc_cli::Result<()> {
		let mut vk_file = File::open(&self.vk_path)?;
		let mut vk_contents = String::new();
		vk_file.read_to_string(&mut vk_contents)?;

		let mut proof_file = File::open(&self.proof_path)?;
		let mut proof_contents = String::new();
		proof_file.read_to_string(&mut proof_contents)?;

		let mut inputs_file = File::open(&self.inputs_path)?;
		let mut inputs_contents = String::new();
		inputs_file.read_to_string(&mut inputs_contents)?;

		let vk = VKey::from_json_u8_slice(vk_contents.as_bytes()).unwrap();
		let proof = Proof::from_json_u8_slice(proof_contents.as_bytes()).unwrap();
		let inputs = deserialize_public_inputs(inputs_contents.as_bytes()).unwrap();

		match verify(
			prepare_verification_key(vk).unwrap(),
			prepare_proof(proof).unwrap(),
			prepare_public_inputs(inputs),
		) {
			Ok(true) => println!("Proof OK"),
			Ok(false) => println!("Proof NOK"),
			Err(_) => println!("Verification error"),
		}
		Ok(())
	}
}
