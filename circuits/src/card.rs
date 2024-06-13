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
struct CardConfig {
    card: Column<Advice>,
    q_range_check: Selector,
}

#[derive(Debug, Clone)]
struct CardChip<F:PrimeField> {
    config: CardConfig,
    _marker: PhantomData<F>,
}

#[derive(Debug, Clone)]
struct CardConstrained<F: PrimeField>
    (AssignedCell<Assigned<F>, F>);

trait CardChipConstrained<F: PrimeField> {
    fn construct(config: CardConfig) -> Self;
    fn configure(meta: &mut ConstraintSystem<F>,
                suite: Column<Advice>) -> CardConfig;
}

impl<F: PrimeField> CardChip<F> {

    fn construct(config: CardConfig) -> Self {
        Self { config, _marker: PhantomData}
    }

    fn configure(meta: &mut ConstraintSystem<F>,
                card: Column<Advice>, q_range_check: Selector) -> CardConfig {

        fn card_check<F: PrimeField> (value: Expression<F>) -> Expression<F> {
            (1..12).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })        
        }
        // card | selector
        //   v        s
        meta.create_gate("card check",
            |meta| {
                let s: Expression<F> = meta.query_selector(q_range_check);
                let v: Expression<F> = meta.query_advice(
                        card, Rotation::cur());

                // Card check [Ace, 2, 3, 4, 5, 6, 7, 8, 9, 10, J, Q, K]
                // v * (1 - v) * (2 - v) * ... * (R - 1 - v)
                //Vec![]
                let check = card_check(v);
                Constraints::with_selector(s, Some(("check", check)))
        });

        CardConfig {
            card: card,
            q_range_check,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>) -> 
        Result<CardConstrained<F>, Error> {

        let offset = 0;

        layouter.assign_region( || "Card", |mut region| {
            self.config.q_range_check.enable(&mut region, offset)?;

            region.assign_advice(|| "card value",
                self.config.card, offset, || value)
                .map(CardConstrained)
        })
    }
}

#[derive(Default)]
struct CardCircuit<F: PrimeField> {
    card: Value<Assigned<F>>,
}

impl<F: PrimeField> Circuit<F> for CardCircuit<F> {

    type Config = CardConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let card = meta.advice_column();
        let q_range_check = meta.selector();

        CardChip::configure(meta, card, q_range_check)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) 
        -> Result<(), Error> {

        let chip: CardChip<F> = CardChip::construct(config);

        let _ = chip.assign(layouter.namespace(|| "Card Assign"),
            self.card);

        Ok(())
    }
}

#[test]
fn test_range_check_1() {
    const k: u32 = 3;

    // Successful case
    let circuit = CardCircuit::<Fp> {
        card: Value::known(Fp::from(12 as u64).into()),
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}