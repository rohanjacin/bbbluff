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
    qty: Column<Advice>,
    suite: Column<Advice>,
    rank: Column<Advice>,
    s_check: Selector,
}

#[derive(Debug, Clone)]
struct CardChip<F:PrimeField> {
    config: CardConfig,
    _marker: PhantomData<F>,
}

#[derive(Debug, Clone)]
struct CardConstrained<F: PrimeField>
    (AssignedCell<Assigned<F>, F>);

impl<F: PrimeField> CardChip<F> {

    fn construct(config: CardConfig) -> Self {
        Self { config, _marker: PhantomData}
    }

    fn configure(meta: &mut ConstraintSystem<F>, qty: Column<Advice>,
                suite: Column<Advice>, rank: Column<Advice>) -> CardConfig {

        let s_check = meta.selector();

        let qty_check = |value: Expression<F>| {
            // Quantity check 1 or 2 or 3 or 4
            (1..4).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })
        };

        let suite_check = |value: Expression<F>| {
            // Suite check Hearts or Spades or Diamonds or Flowers
            (1..4).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })
        };

        let rank_check = |value: Expression<F>| {
            // Rank check Ace or 2 or 3 or 4 or 5 or 6 or 7 or 8 or 9
            //                or 10 or Jack or Queen or King          
            (1..12).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })
        };

        // | a0  |  a1   |  a2  | selector |
        // |-----|-------|------|----------|        
        // | qty | suite | rank | s_check  |
        //  
        meta.create_gate("card check",
            |meta| {
                let s = meta.query_selector(s_check);
                
                // Private inputs
                let qty = meta.query_advice(qty, Rotation::cur());
                let suite = meta.query_advice(suite, Rotation::cur());
                let rank = meta.query_advice(rank, Rotation::cur());

                // Public inputs
/*                let pub_qty = meta.query_advice(pub_qty, Rotation::next());
                let pub_suite = meta.query_advice(pub_suite, Rotation::next());
                let pub_rank = meta.query_advice(pub_rank, Rotation::next());
*/
                // Card check is qty_check AND suite_check AND rank_check
/*                let poly1 = pub_qty - qty.clone(); 
                let poly2 = pub_suite - suite.clone(); 
                let poly3 = pub_rank - rank.clone(); 
*/
                Constraints::with_selector(s,
                        [
                            ("qty_check", qty_check(qty)),
                            ("suite_check", suite_check(suite)),
                            ("rank_check", rank_check(rank)),
/*                            ("poly1", poly1),
                            ("poly2", poly2),
                            ("poly3", poly3),
*/
                        ],
                )
        });

        CardConfig {
            qty,
            suite,
            rank,
/*            pub_qty,
            pub_suite,
            pub_rank,
*/            s_check,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<F>,
        qty: Value<Assigned<F>>, suite: Value<Assigned<F>>,
        rank: Value<Assigned<F>>) -> 
        Result<(), Error> {

        let offset = 0;

        layouter.assign_region( || "Card", |mut region| {
            self.config.s_check.enable(&mut region, offset)?;

            region.assign_advice(|| "qty value",
                self.config.qty, offset, || qty)
                .map(CardConstrained);

            region.assign_advice(|| "suite value",
                self.config.suite, offset, || suite)
                .map(CardConstrained);

            region.assign_advice(|| "rank value",
                self.config.rank, offset, || rank)
                .map(CardConstrained);
            Ok(())
        })
    }
}

#[derive(Default)]
struct CardCircuit<F: PrimeField> {
    qty: Value<Assigned<F>>,
    suite: Value<Assigned<F>>,
    rank: Value<Assigned<F>>,
}

impl<F: PrimeField> Circuit<F> for CardCircuit<F> {

    type Config = CardConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let qty = meta.advice_column();
        let suite = meta.advice_column();
        let rank = meta.advice_column();
/*        let pub_qty = meta.instance_column();
        let pub_suite = meta.instance_column();
        let pub_rank = meta.instance_column();
*/
        CardChip::configure(meta, qty, suite, rank)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) 
        -> Result<(), Error> {

        let chip: CardChip<F> = CardChip::construct(config);

        let _ = chip.assign(layouter.namespace(|| "Card Assign"),
            self.qty, self.suite, self.rank);

        Ok(())
    }
}

#[test]
fn test_range_check_1() {
    const k: u32 = 5;

    // Successful case
    let circuit = CardCircuit::<Fp> {
        qty: Value::known(Fp::from(3 as u64).into()),
        suite: Value::known(Fp::from(2 as u64).into()),
        rank: Value::known(Fp::from(0 as u64).into()),
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}