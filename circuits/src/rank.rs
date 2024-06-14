use std::marker::PhantomData;
use ff::{Field, PrimeField, PrimeFieldBits};

use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value, SimpleFloorPlanner},
    plonk::{Advice, Assigned, Column, ConstraintSystem,
        Constraints, Error, Expression, Selector, Circuit},
    poly::Rotation,
    pasta::Fp,
    dev::{FailureLocation, MockProver, VerifyFailure},    
};

#[derive(Debug, Clone)]
pub struct RankConfig {
    rank: Column<Advice>,
    q_range_check: Selector,
}

#[derive(Debug, Clone)]
pub struct RankChip<F:PrimeField> {
    config: RankConfig,
    _marker: PhantomData<F>,
}

#[derive(Debug, Clone)]
struct RankConstrained<F: PrimeField>
    (AssignedCell<Assigned<F>, F>);

trait RankChipConstrained<F: PrimeField> {
    fn construct(config: RankConfig) -> Self;
    fn configure(meta: &mut ConstraintSystem<F>,
                suite: Column<Advice>) -> RankConfig;
}

impl<F: PrimeField> RankChip<F> {

    pub fn construct(config: RankConfig) -> Self {
        Self { config, _marker: PhantomData}
    }

    pub fn configure(meta: &mut ConstraintSystem<F>,
                rank: Column<Advice>, q_range_check: Selector) -> RankConfig {

        fn rank_check<F: PrimeField> (value: Expression<F>) -> Expression<F> {
            (1..12).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })        
        }
        // rank | selector
        //   v        s
        meta.create_gate("rank check",
            |meta| {
                let s: Expression<F> = meta.query_selector(q_range_check);
                let v: Expression<F> = meta.query_advice(
                        rank, Rotation::cur());

                // Rank check [Ace, 2, 3, 4, 5, 6, 7, 8, 9, 10, J, Q, K]
                // v * (1 - v) * (2 - v) * ... * (R - 1 - v)
                //Vec![]
                let check = rank_check(v);
                Constraints::with_selector(s, Some(("check", check)))
        });

        RankConfig {
            rank: rank,
            q_range_check,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>) -> 
        Result<RankConstrained<F>, Error> {

        let offset = 0;

        layouter.assign_region( || "Rank", |mut region| {
            self.config.q_range_check.enable(&mut region, offset)?;

            region.assign_advice(|| "rank value",
                self.config.rank, offset, || value)
                .map(RankConstrained)
        })
    }
}

#[derive(Default)]
struct RankCircuit<F: PrimeField> {
    rank: Value<Assigned<F>>,
}

impl<F: PrimeField> Circuit<F> for RankCircuit<F> {

    type Config = RankConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let card = meta.advice_column();
        let q_range_check = meta.selector();

        RankChip::configure(meta, card, q_range_check)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) 
        -> Result<(), Error> {

        let chip: RankChip<F> = RankChip::construct(config);

        let _ = chip.assign(layouter.namespace(|| "Rank Assign"),
            self.rank);

        Ok(())
    }
}

#[test]
fn test_range_check_1() {
    const k: u32 = 3;

    // Successful case
    let circuit = RankCircuit::<Fp> {
        rank: Value::known(Fp::from(11 as u64).into()),
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}