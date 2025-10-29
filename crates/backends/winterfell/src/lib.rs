//! Winterfell backend adapter (stub).

use std::convert::TryFrom;

use anyhow::{anyhow, ensure, Context, Result};
use thiserror::Error;
use zkprov_corelib::air::types::{AirIr, CommitmentKind};
use zkprov_corelib::air::AirHash;
use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};
use zkprov_corelib::crypto::registry::hash64_by_id;
use zkprov_corelib::evm::digest::digest_D;
use zkprov_corelib::proof::{self, ProofHeader};

#[derive(Clone, Debug, serde::Serialize)]
pub struct WinterfellCapabilities {
    pub name: &'static str,
    pub field: &'static str,
    pub hashes: Vec<&'static str>,
    pub commitments: Vec<&'static str>,
    pub recursion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub blowup: u8,
    pub fri_arity: u8,
    pub queries: u8,
    pub grinding: u8,
}

pub fn capabilities() -> WinterfellCapabilities {
    WinterfellCapabilities {
        name: BACKEND_ID,
        field: "Prime256",
        hashes: vec!["blake3", "poseidon2", "rescue", "keccak"],
        commitments: vec!["Pedersen(placeholder)", "PoseidonCommit", "KeccakCommit"],
        recursion: false,
    }
}

pub fn profile_map(id: &str) -> Profile {
    match id {
        "fast" | "dev-fast" => Profile {
            blowup: 8,
            fri_arity: 2,
            queries: 24,
            grinding: 16,
        },
        "secure" => Profile {
            blowup: 32,
            fri_arity: 2,
            queries: 50,
            grinding: 20,
        },
        _ => Profile {
            blowup: 16,
            fri_arity: 2,
            queries: 30,
            grinding: 18,
        },
    }
}

#[derive(Debug, Default)]
pub struct WinterfellBackend;

#[derive(Debug)]
pub struct ProveInput<'a> {
    pub ir: &'a AirIr,
    pub profile_id: &'a str,
    pub pub_io_json: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofBytes {
    proof: Vec<u8>,
    header: ProofHeader,
    digest_body: Vec<u8>,
}

impl ProofBytes {
    pub fn proof_bytes(&self) -> &[u8] {
        &self.proof
    }

    pub fn header(&self) -> &ProofHeader {
        &self.header
    }

    pub fn determinism_body(&self) -> &[u8] {
        &self.digest_body
    }

    pub fn digest(&self) -> [u8; 32] {
        digest_D(&self.header, &self.digest_body)
    }
}

const BACKEND_ID: &str = "winterfell@0.6";
const DIGEST_BACKEND_ID: &str = "native@0.0";

fn hash_id_from_air(hash: &AirHash) -> &'static str {
    match hash {
        AirHash::Poseidon2 => "poseidon2",
        AirHash::Blake3 => "blake3",
        AirHash::Rescue => "rescue",
    }
}

fn digest_backend_id(ir: &AirIr) -> &str {
    ir.meta.backend.as_deref().unwrap_or(DIGEST_BACKEND_ID)
}

fn determinism_header(
    ir: &AirIr,
    profile_id: &str,
    pub_io_json: &str,
    body_len: usize,
) -> ProofHeader {
    let backend_id = digest_backend_id(ir);
    ProofHeader {
        backend_id_hash: proof::hash64("BACKEND", backend_id.as_bytes()),
        profile_id_hash: proof::hash64("PROFILE", profile_id.as_bytes()),
        pubio_hash: proof::hash64("PUBIO", pub_io_json.as_bytes()),
        body_len: body_len as u64,
    }
}

fn determinism_manifest_body(
    ir: &AirIr,
    program: &WfProgram,
    pub_io_json: &str,
) -> Result<Vec<u8>> {
    let hash_id = hash_id_from_air(&ir.meta.hash);
    let mut accum = 0u64;

    let mut mix = |label: &str, bytes: &[u8]| -> Result<()> {
        let h = hash64_by_id(hash_id, label, bytes)
            .ok_or_else(|| anyhow!("unsupported hash id '{hash_id}'"))?;
        accum ^= h.rotate_left(13) ^ h.wrapping_mul(0x9e3779b97f4a7c15);
        Ok(())
    };

    mix("AIR.NAME", ir.meta.name.as_bytes())?;
    mix("AIR.FIELD", ir.meta.field.as_bytes())?;

    let rows = u32::try_from(program.trace_rows).context("trace rows exceed u32 range")?;
    let cols = u32::try_from(program.trace_cols).context("trace cols exceed u32 range")?;
    mix("TRACE.ROWS", &rows.to_le_bytes())?;
    mix("TRACE.COLS", &cols.to_le_bytes())?;
    mix("IO.JSON", pub_io_json.as_bytes())?;

    Ok(accum.to_le_bytes().to_vec())
}

fn ensure_commitment_support(ir: &AirIr) -> std::result::Result<(), BackendUnsupported> {
    // Winterfell 0.6 only wires a placeholder Pedersen commitment; reject other curves.
    for curve in ir
        .commitments
        .iter()
        .filter_map(|binding| match &binding.kind {
            CommitmentKind::Pedersen { curve } => Some(curve.clone()),
            _ => None,
        })
    {
        let normalized = if curve.trim().is_empty() {
            "placeholder".to_string()
        } else {
            curve.trim().to_ascii_lowercase()
        };

        if normalized != "placeholder" {
            return Err(BackendUnsupported::PedersenCurve { curve });
        }
    }

    let has_poseidon_commit = ir
        .commitments
        .iter()
        .any(|binding| matches!(binding.kind, CommitmentKind::PoseidonCommit));
    let has_keccak_commit = ir
        .commitments
        .iter()
        .any(|binding| matches!(binding.kind, CommitmentKind::KeccakCommit));

    if !has_poseidon_commit && !has_keccak_commit {
        return Ok(());
    }

    let hash_label = format!("{:?}", ir.meta.hash).to_ascii_lowercase();

    if has_poseidon_commit && hash_label != "poseidon2" {
        return Err(BackendUnsupported::PoseidonCommitHash { hash: hash_label });
    }

    if has_keccak_commit && hash_label != "keccak" {
        return Err(BackendUnsupported::KeccakCommitHash { hash: hash_label });
    }

    Ok(())
}

impl WinterfellBackend {
    pub fn prove(input: ProveInput) -> Result<ProofBytes> {
        let program = to_wf(input.ir)?;
        let profile = profile_map(input.profile_id);

        let proof_bytes = match program.air {
            WfAirKind::Toy(_) => toy::prove(&program, &profile, input.pub_io_json)?,
            other => {
                return Err(unsupported(BackendUnsupported::Other(format!(
                    "Winterfell prover does not yet support '{other:?}' programs"
                ))))
            }
        };

        let digest_body = determinism_manifest_body(input.ir, &program, input.pub_io_json)?;
        let header = determinism_header(
            input.ir,
            input.profile_id,
            input.pub_io_json,
            digest_body.len(),
        );

        Ok(ProofBytes {
            proof: proof_bytes,
            header,
            digest_body,
        })
    }

    pub fn verify(ir: &AirIr, proof: &[u8]) -> Result<()> {
        let program = to_wf(ir)?;

        match program.air {
            WfAirKind::Toy(_) => toy::verify(&program, proof),
            other => Err(unsupported(BackendUnsupported::Other(format!(
                "Winterfell verifier does not yet support '{other:?}' programs"
            )))),
        }
    }
}

impl ProverBackend for WinterfellBackend {
    fn id(&self) -> &'static str {
        BACKEND_ID
    }

    fn capabilities(&self) -> Capabilities {
        let wf_caps = capabilities();
        Capabilities {
            fields: vec![wf_caps.field],
            hashes: wf_caps.hashes.clone(),
            fri_arities: vec![2, 4],
            recursion: if wf_caps.recursion {
                "stark-in-stark"
            } else {
                "none"
            },
            lookups: false,
            curves: vec!["placeholder"],
            pedersen: wf_caps
                .commitments
                .iter()
                .any(|commitment| commitment.starts_with("Pedersen")),
        }
    }
}

impl VerifierBackend for WinterfellBackend {}

mod toy {
    use super::{unsupported, BackendUnsupported, Profile, Result, WfProgram};
    use anyhow::{ensure, Context};
    use serde_json::Value;
    use winterfell::{
        crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree},
        math::{fields::f128::BaseElement, FieldElement, ToElements},
        verify as winterfell_verify, AcceptableOptions, Air, AirContext, Assertion,
        AuxRandElements, BatchingMethod, CompositionPoly, CompositionPolyTrace,
        DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame,
        FieldExtension, Proof, ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable,
        TraceTable, TransitionConstraintDegree,
    };

    use winterfell::PartitionOptions;

    type ToyField = BaseElement;

    const TOY_TRACE_WIDTH: usize = 4;
    const MAX_TOY_TRACE_LENGTH: usize = 1 << 10;

    pub fn prove(program: &WfProgram, profile: &Profile, pub_io_json: &str) -> Result<Vec<u8>> {
        if !pub_io_json.trim().is_empty() {
            serde_json::from_str::<Value>(pub_io_json)
                .context("toy AIR public IO must be valid JSON")?;
        }

        ensure_supported_shape(program)?;

        let options = build_options(profile);
        let trace_length = program
            .trace_rows
            .clamp(TraceInfo::MIN_TRACE_LENGTH, MAX_TOY_TRACE_LENGTH);
        let periodic = build_periodic_values(trace_length);
        let trace = build_trace(trace_length, &periodic);

        let prover = ToyProver::new(options.clone());
        let proof = prover
            .prove(trace)
            .map_err(|err| anyhow::Error::new(err).context("winterfell prover failed"))?;

        Ok(proof.to_bytes())
    }

    pub fn verify(program: &WfProgram, proof: &[u8]) -> Result<()> {
        ensure_supported_shape(program)?;

        let proof = Proof::from_bytes(proof)
            .map_err(|err| anyhow::Error::new(err).context("invalid winterfell proof bytes"))?;
        let acceptable = AcceptableOptions::OptionSet(vec![proof.options().clone()]);

        winterfell_verify::<
            ToyAir,
            Blake3_256<ToyField>,
            DefaultRandomCoin<Blake3_256<ToyField>>,
            MerkleTree<Blake3_256<ToyField>>,
        >(proof, ToyPublicInputs, &acceptable)
        .map_err(|err| anyhow::Error::new(err).context("winterfell verification failed"))
    }

    fn build_trace(length: usize, periodic: &[ToyField]) -> TraceTable<ToyField> {
        let mut trace = TraceTable::new(TOY_TRACE_WIDTH, length);
        let two = ToyField::new(2);

        trace.fill(
            |state| {
                state.fill(ToyField::ZERO);
                if !periodic.is_empty() {
                    state[3] = periodic[0];
                }
            },
            |step, state| {
                state[0] += ToyField::ONE;
                state[1] += two;
                state[2] = state[0] + state[1];
                let next_idx = step + 1;
                if next_idx < periodic.len() {
                    state[3] = periodic[next_idx];
                }
            },
        );

        trace
    }

    fn build_periodic_values(length: usize) -> Vec<ToyField> {
        (0..length)
            .map(|i| {
                if i % 2 == 0 {
                    ToyField::ZERO
                } else {
                    ToyField::ONE
                }
            })
            .collect()
    }

    fn build_options(profile: &Profile) -> ProofOptions {
        let fri_factor = usize::from(profile.fri_arity.max(1));
        let fri_remainder_degree = (fri_factor << 4) - 1;
        ProofOptions::new(
            usize::from(profile.queries),
            usize::from(profile.blowup),
            u32::from(profile.grinding),
            FieldExtension::None,
            fri_factor,
            fri_remainder_degree,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        )
    }

    fn ensure_supported_shape(program: &WfProgram) -> Result<()> {
        ensure!(
            program.trace_cols == TOY_TRACE_WIDTH,
            unsupported(BackendUnsupported::Other(
                "toy prover expects 4 trace columns".into()
            ))
        );
        Ok(())
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct ToyPublicInputs;

    impl ToElements<ToyField> for ToyPublicInputs {
        fn to_elements(&self) -> Vec<ToyField> {
            Vec::new()
        }
    }

    struct ToyAir {
        context: AirContext<ToyField>,
        periodic: Vec<ToyField>,
    }

    impl Air for ToyAir {
        type BaseField = ToyField;
        type PublicInputs = ToyPublicInputs;

        fn new(trace_info: TraceInfo, _pub_inputs: ToyPublicInputs, options: ProofOptions) -> Self {
            let degrees = vec![
                TransitionConstraintDegree::new(1),
                TransitionConstraintDegree::new(1),
                TransitionConstraintDegree::new(1),
            ];
            let periodic = build_periodic_values(trace_info.length());
            Self {
                context: AirContext::new(trace_info, degrees, 2, options),
                periodic,
            }
        }

        fn context(&self) -> &AirContext<Self::BaseField> {
            &self.context
        }

        fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
            vec![self.periodic.clone()]
        }

        fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
            vec![
                Assertion::single(0, 0, ToyField::ZERO),
                Assertion::single(1, 0, ToyField::ZERO),
            ]
        }

        fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
            &self,
            frame: &EvaluationFrame<E>,
            periodic_values: &[E],
            result: &mut [E],
        ) {
            let current = frame.current();
            let next = frame.next();
            let one = E::from(ToyField::ONE);
            let two = E::from(ToyField::new(2));
            let periodic = periodic_values.first().copied().unwrap_or(E::ZERO);

            result[0] = next[0] - current[0] - one;
            result[1] = next[1] - current[1] - two;
            result[2] = current[3] - periodic;
        }
    }

    struct ToyProver {
        options: ProofOptions,
    }

    impl ToyProver {
        fn new(options: ProofOptions) -> Self {
            Self { options }
        }
    }

    impl Prover for ToyProver {
        type BaseField = ToyField;
        type Air = ToyAir;
        type Trace = TraceTable<Self::BaseField>;
        type HashFn = Blake3_256<Self::BaseField>;
        type VC = MerkleTree<Self::HashFn>;
        type RandomCoin = DefaultRandomCoin<Self::HashFn>;
        type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
            DefaultTraceLde<E, Self::HashFn, Self::VC>;
        type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
            DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;
        type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
            DefaultConstraintEvaluator<'a, Self::Air, E>;

        fn get_pub_inputs(&self, _trace: &Self::Trace) -> ToyPublicInputs {
            ToyPublicInputs
        }

        fn options(&self) -> &ProofOptions {
            &self.options
        }

        fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
            &self,
            trace_info: &TraceInfo,
            main_trace: &winterfell::matrix::ColMatrix<Self::BaseField>,
            domain: &StarkDomain<Self::BaseField>,
            partition_option: PartitionOptions,
        ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
            DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
        }

        fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
            &self,
            composition_poly_trace: CompositionPolyTrace<E>,
            num_constraint_composition_columns: usize,
            domain: &StarkDomain<Self::BaseField>,
            partition_options: PartitionOptions,
        ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
            DefaultConstraintCommitment::new(
                composition_poly_trace,
                num_constraint_composition_columns,
                domain,
                partition_options,
            )
        }

        fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
            &self,
            air: &'a Self::Air,
            aux_rand_elements: Option<AuxRandElements<E>>,
            composition_coefficients: winterfell::ConstraintCompositionCoefficients<E>,
        ) -> Self::ConstraintEvaluator<'a, E> {
            DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
        }
    }
}

const DEFAULT_TRACE_ROWS: usize = 1 << 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToyDescriptor {
    pub transition_count: usize,
    pub boundary_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleDescriptor {
    pub hash: AirHash,
    pub arity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WfAirKind {
    Toy(ToyDescriptor),
    Merkle(MerkleDescriptor),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfProgram {
    pub trace_cols: usize,
    pub trace_rows: usize,
    pub const_cols: usize,
    pub periodic_cols: usize,
    pub public_inputs: Vec<u64>,
    pub air: WfAirKind,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BackendUnsupported {
    #[error("Unsupported(program '{program}' not yet supported by Winterfell backend)")]
    Program { program: String },
    #[error(
        "Unsupported(Pedersen commitments require curve 'placeholder' but '{curve}' requested)"
    )]
    PedersenCurve { curve: String },
    #[error(
        "Unsupported(PoseidonCommit requires Winterfell hash 'poseidon2' but '{hash}' requested)"
    )]
    PoseidonCommitHash { hash: String },
    #[error("Unsupported(KeccakCommit requires Winterfell hash 'keccak' but '{hash}' requested)")]
    KeccakCommitHash { hash: String },
    #[error("Unsupported({0})")]
    Other(String),
}

fn unsupported(err: BackendUnsupported) -> anyhow::Error {
    anyhow::Error::new(err)
}

fn convert_toy(ir: &AirIr) -> Result<WfProgram> {
    ensure!(
        ir.columns.trace_cols == 4,
        unsupported(BackendUnsupported::Other(
            "toy AIR expects exactly 4 trace columns".into()
        ))
    );
    ensure!(
        ir.columns.const_cols == 1,
        unsupported(BackendUnsupported::Other(
            "toy AIR expects exactly 1 constant column".into()
        ))
    );
    ensure!(
        ir.columns.periodic_cols == 1,
        unsupported(BackendUnsupported::Other(
            "toy AIR expects exactly 1 periodic column".into()
        ))
    );
    ensure!(
        ir.constraints.transition_count == 3,
        unsupported(BackendUnsupported::Other(
            "toy AIR expects 3 transition constraints".into()
        ))
    );
    ensure!(
        ir.constraints.boundary_count == 2,
        unsupported(BackendUnsupported::Other(
            "toy AIR expects 2 boundary constraints".into()
        ))
    );

    let public_inputs = vec![0; ir.public_inputs.len()];
    Ok(WfProgram {
        trace_cols: ir.columns.trace_cols as usize,
        trace_rows: DEFAULT_TRACE_ROWS,
        const_cols: ir.columns.const_cols as usize,
        periodic_cols: ir.columns.periodic_cols as usize,
        public_inputs,
        air: WfAirKind::Toy(ToyDescriptor {
            transition_count: ir.constraints.transition_count as usize,
            boundary_count: ir.constraints.boundary_count as usize,
        }),
    })
}

fn convert_merkle(ir: &AirIr) -> Result<WfProgram> {
    ensure!(
        ir.columns.const_cols == 0,
        unsupported(BackendUnsupported::Other(
            "merkle AIR must not declare constant columns".into()
        ))
    );
    ensure!(
        ir.columns.periodic_cols == 0,
        unsupported(BackendUnsupported::Other(
            "merkle AIR must not declare periodic columns".into()
        ))
    );
    ensure!(
        ir.columns.trace_cols >= 16,
        unsupported(BackendUnsupported::Other(
            "merkle AIR expects at least 16 trace columns to absorb root".into()
        ))
    );
    ensure!(
        ir.constraints.transition_count >= 1,
        unsupported(BackendUnsupported::Other(
            "merkle AIR requires at least one transition constraint".into()
        ))
    );
    ensure!(
        ir.constraints.boundary_count >= 1,
        unsupported(BackendUnsupported::Other(
            "merkle AIR requires at least one boundary constraint".into()
        ))
    );

    let public_inputs = vec![0; ir.public_inputs.len()];
    Ok(WfProgram {
        trace_cols: ir.columns.trace_cols as usize,
        trace_rows: DEFAULT_TRACE_ROWS,
        const_cols: 0,
        periodic_cols: 0,
        public_inputs,
        air: WfAirKind::Merkle(MerkleDescriptor {
            hash: ir.meta.hash.clone(),
            arity: ir.columns.trace_cols as usize,
        }),
    })
}

pub fn to_wf(ir: &AirIr) -> Result<WfProgram> {
    ensure_commitment_support(ir).map_err(unsupported)?;

    match ir.meta.name.as_str() {
        name if name.starts_with("toy") => convert_toy(ir),
        name if name.contains("merkle") => convert_merkle(ir),
        other => Err(unsupported(BackendUnsupported::Program {
            program: other.to_string(),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkprov_backend_native::native_prove;
    use zkprov_corelib::air::parser::parse_air_str;
    use zkprov_corelib::config::Config;

    fn minimal_air(hash: &str) -> String {
        format!(
            r#"
[meta]
name = "demo"
field = "Prime254"
hash = "{hash}"

[columns]
trace_cols = 4
const_cols = 0
periodic_cols = 0

[constraints]
transition_count = 1
boundary_count = 1

[[public_inputs]]
name = "x"
type = "field"

[[public_inputs]]
name = "acc"
type = "bytes"

[[public_inputs]]
name = "digest"
type = "u64"
"#
        )
    }

    fn minimal_air_with_section(hash: &str, section: &str) -> String {
        format!("{}\n{}\n", minimal_air(hash), section)
    }

    #[test]
    fn exposes_capabilities() {
        let caps = capabilities();
        assert_eq!(caps.name, "winterfell@0.6");
        assert_eq!(caps.field, "Prime256");
        assert_eq!(caps.hashes, vec!["blake3", "poseidon2", "rescue", "keccak"]);
        assert!(!caps.recursion);
    }

    #[test]
    fn profiles_have_reasonable_defaults() {
        let fast = profile_map("fast");
        let balanced = profile_map("balanced");
        let secure = profile_map("secure");

        assert!(fast.blowup < balanced.blowup);
        assert!(secure.blowup > balanced.blowup);
        assert_eq!(balanced.fri_arity, 2);
        assert_eq!(profile_map("unknown").blowup, balanced.blowup);
    }

    #[test]
    fn converts_toy_air_to_winterfell_program() {
        let src = include_str!("../../../../examples/air/toy.air");
        let ir = parse_air_str(src).expect("parse toy AIR");

        let wf = to_wf(&ir).expect("convert toy AIR");
        assert_eq!(wf.trace_cols, 4);
        assert_eq!(wf.const_cols, 1);
        assert_eq!(wf.periodic_cols, 1);
        assert_eq!(wf.trace_rows, DEFAULT_TRACE_ROWS);
        assert_eq!(wf.public_inputs.len(), ir.public_inputs.len());

        match wf.air {
            WfAirKind::Toy(ref toy) => {
                assert_eq!(toy.transition_count, 3);
                assert_eq!(toy.boundary_count, 2);
            }
            _ => panic!("expected toy descriptor"),
        }
    }

    #[test]
    fn rejects_unknown_program() {
        let src = r#"
            [meta]
            name = "unknown"
            field = "Prime254"
            hash = "blake3"

            [columns]
            trace_cols = 4

            [constraints]
            transition_count = 1
            boundary_count = 1
        "#;
        let mut ir = parse_air_str(src).expect("parse minimal AIR");
        ir.meta.name = "unknown".to_string();

        let err = to_wf(&ir).expect_err("should reject unsupported program");
        let msg = format!("{err}");
        assert!(msg.contains("Unsupported"));
    }

    #[test]
    fn rejects_non_placeholder_pedersen_curve() {
        let src = minimal_air_with_section(
            "poseidon2",
            r#"[commitments]
pedersen = { curve = "pallas", public = ["x"] }
"#,
        );
        let ir = parse_air_str(&src).expect("parse pedersen AIR");

        let err = to_wf(&ir).expect_err("should reject non-placeholder curve");
        let cause = err
            .downcast_ref::<BackendUnsupported>()
            .expect("pedersen curve unsupported");
        assert_eq!(
            cause,
            &BackendUnsupported::PedersenCurve {
                curve: "pallas".to_string(),
            }
        );
    }

    #[test]
    fn rejects_poseidon_commit_when_hash_mismatch() {
        let src = minimal_air_with_section(
            "blake3",
            r#"[commitments]
poseidon_commit = { public = ["acc"] }
"#,
        );
        let ir = parse_air_str(&src).expect("parse poseidon AIR");

        let err = to_wf(&ir).expect_err("should reject poseidon hash mismatch");
        let cause = err
            .downcast_ref::<BackendUnsupported>()
            .expect("poseidon commit unsupported");
        assert_eq!(
            cause,
            &BackendUnsupported::PoseidonCommitHash {
                hash: "blake3".to_string(),
            }
        );
    }

    #[test]
    fn rejects_keccak_commit_when_hash_mismatch() {
        let src = minimal_air_with_section(
            "poseidon2",
            r#"[commitments]
keccak_commit = { public = ["digest"] }
"#,
        );
        let ir = parse_air_str(&src).expect("parse keccak AIR");

        let err = to_wf(&ir).expect_err("should reject keccak hash mismatch");
        let cause = err
            .downcast_ref::<BackendUnsupported>()
            .expect("keccak commit unsupported");
        assert_eq!(
            cause,
            &BackendUnsupported::KeccakCommitHash {
                hash: "poseidon2".to_string(),
            }
        );
    }

    #[test]
    fn proves_and_verifies_toy_air() {
        let src = include_str!("../../../../examples/air/toy.air");
        let ir = parse_air_str(src).expect("parse toy AIR");

        let proof = WinterfellBackend::prove(ProveInput {
            ir: &ir,
            profile_id: "balanced",
            pub_io_json: "{}",
        })
        .expect("winterfell proof generation");

        WinterfellBackend::verify(&ir, proof.proof_bytes()).expect("winterfell verification");
    }

    fn native_digest_for_air(
        ir: &AirIr,
        inputs: &str,
        air_path: &str,
        hash: &str,
        profile: &str,
    ) -> [u8; 32] {
        let backend_id = digest_backend_id(ir);
        let cfg = Config::new(backend_id, "Prime254", hash, 2, false, profile);
        let proof = native_prove(&cfg, inputs, air_path).expect("native prove");
        let header = ProofHeader::decode(&proof[0..40]).expect("decode header");
        let body = &proof[40..];
        digest_D(&header, body)
    }

    #[test]
    fn digest_matches_native_for_toy_demo() {
        let air_src = include_str!("../../../../examples/air/toy.air");
        let air_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../examples/air/toy.air");
        let ir = parse_air_str(air_src).expect("parse toy AIR");

        let proof = WinterfellBackend::prove(ProveInput {
            ir: &ir,
            profile_id: "balanced",
            pub_io_json: "{}",
        })
        .expect("winterfell proof generation");

        let wf_digest = proof.digest();
        let native_digest = native_digest_for_air(&ir, "{}", air_path, "blake3", "balanced");

        assert_eq!(wf_digest, native_digest);
    }

    #[test]
    fn digest_matches_native_for_secure_profile() {
        let air_src = include_str!("../../../../examples/air/toy.air");
        let air_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../examples/air/toy.air");
        let ir = parse_air_str(air_src).expect("parse toy AIR");

        let proof = WinterfellBackend::prove(ProveInput {
            ir: &ir,
            profile_id: "secure",
            pub_io_json: "{}",
        })
        .expect("winterfell proof generation");

        let wf_digest = proof.digest();
        let native_digest = native_digest_for_air(&ir, "{}", air_path, "blake3", "secure");

        assert_eq!(wf_digest, native_digest);
    }
}
