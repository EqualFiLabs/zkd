//! Winterfell backend adapter (stub).

use anyhow::{ensure, Result};
use thiserror::Error;
use zkprov_corelib::air::types::AirIr;
use zkprov_corelib::air::AirHash;
use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};

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
        name: "winterfell@0.6",
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
pub struct ProofBytes(pub Vec<u8>);

impl WinterfellBackend {
    pub fn prove(input: ProveInput) -> Result<ProofBytes> {
        let program = to_wf(input.ir)?;
        let profile = profile_map(input.profile_id);

        match program.air {
            WfAirKind::Toy(_) => toy::prove(&program, &profile, input.pub_io_json),
            other => Err(unsupported(format!(
                "Winterfell prover does not yet support '{other:?}' programs"
            ))),
        }
    }

    pub fn verify(ir: &AirIr, proof: &[u8]) -> Result<()> {
        let program = to_wf(ir)?;

        match program.air {
            WfAirKind::Toy(_) => toy::verify(&program, proof),
            other => Err(unsupported(format!(
                "Winterfell verifier does not yet support '{other:?}' programs"
            ))),
        }
    }
}

impl ProverBackend for WinterfellBackend {
    fn id(&self) -> &'static str {
        "winterfell@0.6"
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
    use super::{unsupported, Profile, ProofBytes, Result, WfProgram};
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

    pub fn prove(program: &WfProgram, profile: &Profile, pub_io_json: &str) -> Result<ProofBytes> {
        if !pub_io_json.trim().is_empty() {
            serde_json::from_str::<Value>(pub_io_json)
                .context("toy AIR public IO must be valid JSON")?;
        }

        ensure_supported_shape(program)?;

        let options = build_options(profile);
        let trace_length =
            program
                .trace_rows
                .clamp(TraceInfo::MIN_TRACE_LENGTH, MAX_TOY_TRACE_LENGTH);
        let periodic = build_periodic_values(trace_length);
        let trace = build_trace(trace_length, &periodic);

        let prover = ToyProver::new(options.clone());
        let proof = prover
            .prove(trace)
            .map_err(|err| anyhow::Error::new(err).context("winterfell prover failed"))?;

        Ok(ProofBytes(proof.to_bytes()))
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
            unsupported("toy prover expects 4 trace columns")
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

#[derive(Debug, Error)]
#[error("Unsupported({0})")]
struct UnsupportedError(String);

fn unsupported(msg: impl Into<String>) -> anyhow::Error {
    UnsupportedError(msg.into()).into()
}

fn convert_toy(ir: &AirIr) -> Result<WfProgram> {
    ensure!(
        ir.columns.trace_cols == 4,
        unsupported("toy AIR expects exactly 4 trace columns")
    );
    ensure!(
        ir.columns.const_cols == 1,
        unsupported("toy AIR expects exactly 1 constant column")
    );
    ensure!(
        ir.columns.periodic_cols == 1,
        unsupported("toy AIR expects exactly 1 periodic column")
    );
    ensure!(
        ir.constraints.transition_count == 3,
        unsupported("toy AIR expects 3 transition constraints")
    );
    ensure!(
        ir.constraints.boundary_count == 2,
        unsupported("toy AIR expects 2 boundary constraints")
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
        unsupported("merkle AIR must not declare constant columns")
    );
    ensure!(
        ir.columns.periodic_cols == 0,
        unsupported("merkle AIR must not declare periodic columns")
    );
    ensure!(
        ir.columns.trace_cols >= 16,
        unsupported("merkle AIR expects at least 16 trace columns to absorb root")
    );
    ensure!(
        ir.constraints.transition_count >= 1,
        unsupported("merkle AIR requires at least one transition constraint")
    );
    ensure!(
        ir.constraints.boundary_count >= 1,
        unsupported("merkle AIR requires at least one boundary constraint")
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
    match ir.meta.name.as_str() {
        name if name.starts_with("toy") => convert_toy(ir),
        name if name.contains("merkle") => convert_merkle(ir),
        other => Err(unsupported(format!(
            "program '{other}' not supported by Winterfell backend"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkprov_corelib::air::parser::parse_air_str;

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
    fn proves_and_verifies_toy_air() {
        let src = include_str!("../../../../examples/air/toy.air");
        let ir = parse_air_str(src).expect("parse toy AIR");

        let proof = WinterfellBackend::prove(ProveInput {
            ir: &ir,
            profile_id: "balanced",
            pub_io_json: "{}",
        })
        .expect("winterfell proof generation");

        WinterfellBackend::verify(&ir, &proof.0).expect("winterfell verification");
    }
}
