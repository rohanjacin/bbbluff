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
struct QtyConfig {
    qty: Column<Advice>,
    q_range_check: Selector,
}

#[derive(Debug, Clone)]
struct QtyChip<F:PrimeField> {
    config: QtyConfig,
    _marker: PhantomData<F>,
}

#[derive(Debug, Clone)]
struct QtyConstrained<F: PrimeField>
    (AssignedCell<Assigned<F>, F>);

trait QtyChipConstrained<F: PrimeField> {
    fn construct(config: QtyConfig) -> Self;
    fn configure(meta: &mut ConstraintSystem<F>,
                suite: Column<Advice>) -> QtyConfig;
}

impl<F: PrimeField> QtyChip<F> {

    fn construct(config: QtyConfig) -> Self {
        Self { config, _marker: PhantomData}
    }

    fn configure(meta: &mut ConstraintSystem<F>,
                qty: Column<Advice>, q_range_check: Selector) -> QtyConfig {

        fn card_check<F: PrimeField> (value: Expression<F>) -> Expression<F> {
            (1..4).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })        
        }
        // qty | selector
        //   v        s
        meta.create_gate("qty check",
            |meta| {
                let s: Expression<F> = meta.query_selector(q_range_check);
                let v: Expression<F> = meta.query_advice(
                        qty, Rotation::cur());

                // Qty check [1, 2, 3, 4]
                // v * (1 - v) * (2 - v) * ... * (R - 1 - v)
                //Vec![]
                let check = card_check(v);
                Constraints::with_selector(s, Some(("check", check)))
        });

        QtyConfig {
            qty: qty,
            q_range_check,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>) -> 
        Result<QtyConstrained<F>, Error> {

        let offset = 0;

        layouter.assign_region( || "Qty", |mut region| {
            self.config.q_range_check.enable(&mut region, offset)?;

            region.assign_advice(|| "qty value",
                self.config.qty, offset, || value)
                .map(QtyConstrained)
        })
    }
}

#[derive(Default)]
struct QtyCircuit<F: PrimeField> {
    qty: Value<Assigned<F>>,
}

impl<F: PrimeField> Circuit<F> for QtyCircuit<F> {

    type Config = QtyConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let qty = meta.advice_column();
        let q_range_check = meta.selector();

        QtyChip::configure(meta, qty, q_range_check)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) 
        -> Result<(), Error> {

        let chip: QtyChip<F> = QtyChip::construct(config);

        let _ = chip.assign(layouter.namespace(|| "Qty Assign"),
            self.qty);

        Ok(())
    }
}

#[test]
fn test_range_check_1() {
    const k: u32 = 3;

    // Successful case
    let circuit = QtyCircuit::<Fp> {
        qty: Value::known(Fp::from(0 as u64).into()),
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}