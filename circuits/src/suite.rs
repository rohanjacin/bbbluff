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
struct SuiteConfig {
    suite: Column<Advice>,
    q_range_check: Selector,
}

#[derive(Debug, Clone)]
struct SuiteChip<F:PrimeField> {
    config: SuiteConfig,
    _marker: PhantomData<F>,
}

#[derive(Debug, Clone)]
struct SuiteConstrained<F: PrimeField>
    (AssignedCell<Assigned<F>, F>);

trait SuiteChipConstrained<F: PrimeField> {
    fn construct(config: SuiteConfig) -> Self;
    fn configure(meta: &mut ConstraintSystem<F>,
                suite: Column<Advice>) -> SuiteConfig;
}

impl<F: PrimeField> SuiteChip<F> {

    fn construct(config: SuiteConfig) -> Self {
        Self { config, _marker: PhantomData}
    }

    fn configure(meta: &mut ConstraintSystem<F>,
                suite: Column<Advice>, q_range_check: Selector) -> SuiteConfig {

        fn suite_check<F: PrimeField> (value: Expression<F>) -> Expression<F> {
            //value.clone() * (Expression::Constant(Fp::from(1  as u64)) - value.clone())
            (1..4).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })        
        }
        // suite | selector
        //   v        s
        meta.create_gate("suite check",
            |meta| {
                let s: Expression<F> = meta.query_selector(q_range_check);
                let v: Expression<F> = meta.query_advice(
                        suite, Rotation::cur());

                // Suite check [1-Hearts, 2-Diamonds, 3-Spades, 4-Flowers]
                // v * (1 - v) * (2 - v) * ... * (R - 1 - v)
                //Vec![]
                let check = suite_check(v);
                Constraints::with_selector(s, Some(("check", check)))
        });

        SuiteConfig {
            suite: suite,
            q_range_check,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>) -> 
        Result<SuiteConstrained<F>, Error> {

        let offset = 0;

        layouter.assign_region( || "Suite", |mut region| {
            self.config.q_range_check.enable(&mut region, offset)?;

            region.assign_advice(|| "suite value",
                self.config.suite, offset, || value)
                .map(SuiteConstrained)
        })
    }
}


#[derive(Default)]
struct SuiteCircuit<F: PrimeField> {
    suite: Value<Assigned<F>>,
}

impl<F: PrimeField> Circuit<F> for SuiteCircuit<F> {

    type Config = SuiteConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let suite = meta.advice_column();
        let q_range_check = meta.selector();

        SuiteChip::configure(meta, suite, q_range_check)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) 
        -> Result<(), Error> {

        let chip: SuiteChip<F> = SuiteChip::construct(config);

        let _ = chip.assign(layouter.namespace(|| "Suite Assign"),
            self.suite);

        Ok(())
    }
}

#[test]
fn test_range_check_1() {
    const k: u32 = 3;

    // Successful case
    let circuit = SuiteCircuit::<Fp> {
        suite: Value::known(Fp::from(0 as u64).into()),
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}

#[cfg(feature = "dev-graph")]
#[test]
fn print_range_check_1() {
    use plotters::prelude::*;

    let root = BitMapBackend::new("range-check-1-layout.png", (1024, 3096)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Range Check 1 Layout", ("sans-serif", 60))
        .unwrap();

    let circuit = SuiteCircuit::<Fp> {
        suite: Value::known(Fp::from(0 as u64).into()),
    };
    halo2_proofs::dev::CircuitLayout::default()
        .render(3, &circuit, &root)
        .unwrap();
}